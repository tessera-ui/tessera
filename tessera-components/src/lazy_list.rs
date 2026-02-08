//! Virtualized list components for displaying long scrolling feeds.
//!
//! ## Usage
//!
//! Use `lazy_column` or `lazy_row` to efficiently display large datasets.
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
    sync::Arc,
};

use derive_setters::Setters;
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, NodeId,
    ParentConstraint, Px, PxPosition, State, key,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera,
};

use crate::{
    alignment::CrossAxisAlignment,
    scrollable::{ScrollableArgs, ScrollableController, scrollable_with_controller},
};

const DEFAULT_VIEWPORT_ITEMS: usize = 8;

/// Persistent state for lazy list components.
///
/// This controller holds both the scroll position ([`ScrollableController`])
/// and the item measurement cache. When retained across navigation (via
/// [`retain`] or [`retain_with_key`]), both the scroll position and cached
/// measurements will be preserved, ensuring correct layout when remounting.
///
/// [`retain`]: tessera_ui::retain
/// [`retain_with_key`]: tessera_ui::retain_with_key
///
/// # Examples
///
/// ```
/// use tessera_components::lazy_list::{
///     LazyColumnArgs, LazyListController, lazy_column_with_controller,
/// };
/// use tessera_ui::{retain_with_key, tessera};
///
/// #[tessera]
/// fn scrollable_page(page_id: &str) {
///     // Both scroll position and measurement cache persist across navigation
///     let controller = retain_with_key(page_id, LazyListController::new);
///     lazy_column_with_controller(LazyColumnArgs::default(), controller, |scope| {
///         scope.items(100, |i| { /* ... */ });
///     });
/// }
/// ```
pub struct LazyListController {
    scroll: ScrollableController,
    cache: LazyListCache,
}

impl Default for LazyListController {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyListController {
    /// Creates a new lazy list controller with default scroll position and
    /// empty cache.
    pub fn new() -> Self {
        Self {
            scroll: ScrollableController::new(),
            cache: LazyListCache::default(),
        }
    }

    /// Returns a reference to the underlying scroll controller.
    pub fn scroll_controller(&self) -> &ScrollableController {
        &self.scroll
    }

    /// Returns a mutable reference to the underlying scroll controller.
    pub fn scroll_controller_mut(&mut self) -> &mut ScrollableController {
        &mut self.scroll
    }
}

/// Arguments shared between lazy lists.
#[derive(Clone, Setters)]
pub struct LazyColumnArgs {
    /// Modifier for the scroll container.
    pub modifier: Modifier,
    /// How children are aligned along the cross axis (horizontal for columns).
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Gap between successive items.
    pub item_spacing: Dp,
    /// Number of extra items instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each item, used before real measurements
    /// exist.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the lazy list content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents. Prevents gigantic
    /// textures when nesting the list inside wrap/auto-sized surfaces.
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyColumnArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            cross_axis_alignment: CrossAxisAlignment::Start,
            item_spacing: Dp(0.0),
            overscan: 2,
            estimated_item_size: Dp(48.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
        }
    }
}

/// Arguments for `lazy_row`. Identical to [`LazyColumnArgs`] but horizontal
/// scrolling is enforced.
#[derive(Clone, Setters)]
pub struct LazyRowArgs {
    /// Modifier for the scroll container.
    pub modifier: Modifier,
    /// How children are aligned along the cross axis (vertical for rows).
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Gap between successive items.
    pub item_spacing: Dp,
    /// Number of extra items instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each item, used before real measurements
    /// exist.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the lazy list content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents for horizontal lists.
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyRowArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            cross_axis_alignment: CrossAxisAlignment::Start,
            item_spacing: Dp(0.0),
            overscan: 2,
            estimated_item_size: Dp(48.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
        }
    }
}

/// Scope used to add lazily generated children to a lazy list.
pub struct LazyListScope<'a> {
    slots: &'a mut Vec<LazySlot>,
}

impl<'a> LazyListScope<'a> {
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

    /// Adds a sticky header item that remains pinned while scrolling.
    pub fn sticky_header<F>(&mut self, builder: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.slots.push(LazySlot::sticky(builder, None));
    }

    /// Adds a sticky header item with a stable key.
    pub fn sticky_header_with_key<K, F>(&mut self, key: K, builder: F)
    where
        K: Hash,
        F: Fn() + Send + Sync + 'static,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let key_hash = hasher.finish();
        self.slots.push(LazySlot::sticky(builder, Some(key_hash)));
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

/// Scope alias for vertical lazy lists.
pub type LazyColumnScope<'a> = LazyListScope<'a>;
/// Scope alias for horizontal lazy lists.
pub type LazyRowScope<'a> = LazyListScope<'a>;

/// # lazy_column
///
/// A vertically scrolling list that only renders items visible in the viewport.
///
/// ## Usage
///
/// Display a long, vertical list of items without incurring the performance
/// cost of rendering every item at once.
///
/// ## Parameters
///
/// - `args` — configures the list's layout and scrolling behavior; see
///   [`LazyColumnArgs`].
/// - `configure` — a closure that receives a [`LazyColumnScope`] for adding
///   items to the list.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_list::{LazyColumnArgs, lazy_column},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     lazy_column(LazyColumnArgs::default(), |scope| {
///         scope.items(1000, |i| {
///             let text_content = format!("Item #{i}");
///             text(TextArgs::default().text(text_content));
///         });
///     });
/// }
/// ```
#[tessera]
pub fn lazy_column<F>(args: LazyColumnArgs, configure: F)
where
    F: FnOnce(&mut LazyColumnScope),
{
    let controller = remember(LazyListController::new);
    lazy_column_with_controller(args, controller, configure);
}

/// # lazy_column_with_controller
///
/// Controlled lazy column variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to preserve scroll position across navigation or share
/// state between components. Pass a [`LazyListController`] created with
/// [`retain`] or [`retain_with_key`] to persist both scroll position and
/// measurement cache.
///
/// [`retain`]: tessera_ui::retain
/// [`retain_with_key`]: tessera_ui::retain_with_key
///
/// ## Parameters
///
/// - `args` — configures the list's layout and scrolling behavior; see
///   [`LazyColumnArgs`].
/// - `controller` — a [`LazyListController`] that holds scroll position and
///   item measurement cache.
/// - `configure` — a closure that receives a [`LazyColumnScope`] for adding
///   items to the list.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(LazyListController::new);
///     lazy_column_with_controller(LazyColumnArgs::default(), controller, |scope| {
///         scope.items(5, |i| {
///             let text_content = format!("Row #{i}");
///             text(TextArgs::default().text(text_content));
///         });
///     });
/// }
/// ```
#[tessera]
pub fn lazy_column_with_controller<F>(
    args: LazyColumnArgs,
    controller: State<LazyListController>,
    configure: F,
) where
    F: FnOnce(&mut LazyColumnScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyColumnScope { slots: &mut slots };
        configure(&mut scope);
    }

    let scrollable_args = ScrollableArgs::default()
        .modifier(args.modifier)
        .vertical(true)
        .horizontal(false);

    let view_args = LazyListViewArgs {
        axis: LazyListAxis::Vertical,
        cross_axis_alignment: args.cross_axis_alignment,
        item_spacing: sanitize_spacing(Px::from(args.item_spacing)),
        estimated_item_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
    };

    // Create a proxy scroll controller that syncs with the LazyListController
    let scroll_controller = remember(ScrollableController::default);

    // Restore saved position from controller on first mount
    let saved_position = controller.with(|c| c.scroll.child_position());
    scroll_controller.with_mut(|sc| {
        if sc.child_position() == PxPosition::ZERO && saved_position != PxPosition::ZERO {
            sc.set_scroll_position(saved_position);
        }
    });

    scrollable_with_controller(scrollable_args, scroll_controller, move || {
        // Sync scroll position back to controller
        let current_pos = scroll_controller.with(|sc| sc.child_position());
        controller.with_mut(|c| c.scroll.set_scroll_position(current_pos));

        lazy_list_view(view_args, controller, slots.clone(), scroll_controller);
    });
}

/// # lazy_row
///
/// A horizontally scrolling list that only renders items visible in the
/// viewport.
///
/// ## Usage
///
/// Display a long, horizontal list of items, such as a gallery or a set of
/// chips.
///
/// ## Parameters
///
/// - `args` — configures the list's layout and scrolling behavior; see
///   [`LazyRowArgs`].
/// - `configure` — a closure that receives a [`LazyRowScope`] for adding items
///   to the list.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_list::{LazyRowArgs, lazy_row},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     lazy_row(LazyRowArgs::default(), |scope| {
///         scope.items(100, |i| {
///             let text_content = format!("Item {i}");
///             text(TextArgs::default().text(text_content));
///         });
///     });
/// }
/// ```
#[tessera]
pub fn lazy_row<F>(args: LazyRowArgs, configure: F)
where
    F: FnOnce(&mut LazyRowScope),
{
    let controller = remember(LazyListController::new);
    lazy_row_with_controller(args, controller, configure);
}

/// # lazy_row_with_controller
///
/// Controlled lazy row variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to preserve scroll position across navigation or share
/// state between components. Pass a [`LazyListController`] created with
/// [`retain`] or [`retain_with_key`] to persist both scroll position and
/// measurement cache.
///
/// [`retain`]: tessera_ui::retain
/// [`retain_with_key`]: tessera_ui::retain_with_key
///
/// ## Parameters
///
/// - `args` — configures the list's layout and scrolling behavior; see
///   [`LazyRowArgs`].
/// - `controller` — a [`LazyListController`] that holds scroll position and
///   item measurement cache.
/// - `configure` — a closure that receives a [`LazyRowScope`] for adding items
///   to the list.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_list::{LazyListController, LazyRowArgs, lazy_row_with_controller},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(LazyListController::new);
///     lazy_row_with_controller(LazyRowArgs::default(), controller, |scope| {
///         scope.items(3, |i| {
///             let text_content = format!("Card {i}");
///             text(TextArgs::default().text(text_content));
///         });
///     });
/// }
/// ```
#[tessera]
pub fn lazy_row_with_controller<F>(
    args: LazyRowArgs,
    controller: State<LazyListController>,
    configure: F,
) where
    F: FnOnce(&mut LazyRowScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyRowScope { slots: &mut slots };
        configure(&mut scope);
    }

    let scrollable_args = ScrollableArgs::default()
        .modifier(args.modifier)
        .vertical(false)
        .horizontal(true);

    let view_args = LazyListViewArgs {
        axis: LazyListAxis::Horizontal,
        cross_axis_alignment: args.cross_axis_alignment,
        item_spacing: sanitize_spacing(Px::from(args.item_spacing)),
        estimated_item_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
    };

    // Create a proxy scroll controller that syncs with the LazyListController
    let scroll_controller = remember(ScrollableController::default);

    // Restore saved position from controller on first mount
    let saved_position = controller.with(|c| c.scroll.child_position());
    scroll_controller.with_mut(|sc| {
        if sc.child_position() == PxPosition::ZERO && saved_position != PxPosition::ZERO {
            sc.set_scroll_position(saved_position);
        }
    });

    scrollable_with_controller(scrollable_args, scroll_controller, move || {
        // Sync scroll position back to controller
        let current_pos = scroll_controller.with(|sc| sc.child_position());
        controller.with_mut(|c| c.scroll.set_scroll_position(current_pos));

        lazy_list_view(view_args, controller, slots.clone(), scroll_controller);
    });
}

#[derive(Clone)]
struct LazyListViewArgs {
    axis: LazyListAxis,
    cross_axis_alignment: CrossAxisAlignment,
    item_spacing: Px,
    estimated_item_main: Px,
    overscan: usize,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
}

#[tessera]
fn lazy_list_view(
    view_args: LazyListViewArgs,
    controller: State<LazyListController>,
    slots: Vec<LazySlot>,
    scroll_controller: State<ScrollableController>,
) {
    let plan = LazySlotPlan::new(slots);
    let total_count = plan.total_count();

    controller.with_mut(|c| c.cache.set_item_count(total_count));

    let scroll_offset = view_args
        .axis
        .scroll_offset(scroll_controller.with(|s| s.child_position()));
    let padding_main = view_args.padding_main;
    let viewport_span = resolve_viewport_span(
        view_args
            .axis
            .visible_span(scroll_controller.with(|s| s.visible_size())),
        view_args.estimated_item_main,
        view_args.item_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);
    let total_main = controller.with(|c| {
        c.cache
            .total_main_size(view_args.estimated_item_main, view_args.item_spacing)
    });
    let total_main_with_padding = total_main + padding_main + padding_main;
    let visible_cross = view_args
        .axis
        .cross(&scroll_controller.with(|s| s.visible_size()));
    let cross_with_padding = visible_cross + view_args.padding_cross + view_args.padding_cross;
    scroll_controller.with_mut(|c| {
        c.override_child_size(
            view_args
                .axis
                .pack_size(total_main_with_padding, cross_with_padding),
        );
    });

    let visible_children = controller.with(|c| {
        compute_visible_children(
            &plan,
            &c.cache,
            total_count,
            scroll_offset,
            viewport_span,
            view_args.overscan,
            view_args.estimated_item_main,
            view_args.item_spacing,
        )
    });

    if visible_children.is_empty() {
        layout(ZeroLayout);
        return;
    }

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_item_indices = visible_children
        .iter()
        .map(|visible| visible.item_index)
        .collect();

    layout(LazyListLayout {
        axis: view_args.axis,
        cross_axis_alignment: view_args.cross_axis_alignment,
        item_spacing: view_args.item_spacing,
        estimated_item_main: view_args.estimated_item_main,
        max_viewport_main: view_args.max_viewport_main,
        padding_main,
        padding_cross: view_args.padding_cross,
        viewport_limit,
        visible_item_indices,
        sticky_indices: plan.sticky_indices().to_vec(),
        scroll_offset,
        controller,
        scroll_controller,
    });

    for child in visible_children {
        key(child.key_hash, || {
            (child.builder)(child.local_index);
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Clone)]
struct LazyListLayout {
    axis: LazyListAxis,
    cross_axis_alignment: CrossAxisAlignment,
    item_spacing: Px,
    estimated_item_main: Px,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
    viewport_limit: Px,
    visible_item_indices: Vec<usize>,
    sticky_indices: Vec<usize>,
    scroll_offset: Px,
    controller: State<LazyListController>,
    scroll_controller: State<ScrollableController>,
}

impl PartialEq for LazyListLayout {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.item_spacing == other.item_spacing
            && self.estimated_item_main == other.estimated_item_main
            && self.max_viewport_main == other.max_viewport_main
            && self.padding_main == other.padding_main
            && self.padding_cross == other.padding_cross
            && self.viewport_limit == other.viewport_limit
            && self.visible_item_indices == other.visible_item_indices
            && self.sticky_indices == other.sticky_indices
            && self.scroll_offset == other.scroll_offset
    }
}

impl LayoutSpec for LazyListLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        if input.children_ids().len() != self.visible_item_indices.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy list measured child count mismatch".into(),
            ));
        }

        let mut measured_children: Vec<(usize, NodeId)> = self
            .visible_item_indices
            .iter()
            .copied()
            .zip(input.children_ids().iter().copied())
            .collect();
        measured_children.sort_unstable_by_key(|(item_index, _)| *item_index);

        let mut child_constraint = self.axis.child_constraint(input.parent_constraint());
        apply_cross_padding(&mut child_constraint, self.axis, self.padding_cross);
        let (placements, inner_cross, total_main) = self.controller.with_mut(|c| {
            let mut placements = Vec::with_capacity(self.visible_item_indices.len());
            let mut max_cross = Px::ZERO;

            for (item_index, child_id) in &measured_children {
                let item_offset =
                    c.cache
                        .offset_for(*item_index, self.estimated_item_main, self.item_spacing);
                let child_size = input.measure_child(*child_id, &child_constraint)?;

                c.cache.record_measurement(
                    *item_index,
                    self.axis.main(&child_size),
                    self.estimated_item_main,
                );

                max_cross = max_cross.max(self.axis.cross(&child_size));
                placements.push(Placement {
                    item_index: *item_index,
                    child_id: *child_id,
                    offset_main: item_offset,
                    size: child_size,
                });
            }

            let total_main = c
                .cache
                .total_main_size(self.estimated_item_main, self.item_spacing);
            Ok::<_, MeasurementError>((placements, max_cross, total_main))
        })?;

        let total_main_with_padding = total_main + self.padding_main + self.padding_main;
        let cross_with_padding = inner_cross + self.padding_cross + self.padding_cross;
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

        for placement in &placements {
            let cross_offset = compute_cross_offset(
                inner_cross,
                self.axis.cross(&placement.size),
                self.cross_axis_alignment,
            );
            let mut main_offset = placement.offset_main + self.padding_main;
            if self.is_sticky(placement.item_index) {
                let sticky_start = self.scroll_offset + self.padding_main;
                main_offset = main_offset.max(sticky_start);
                if let Some(next_index) = self.next_sticky_after(placement.item_index) {
                    let next_offset = self.controller.with(|c| {
                        c.cache
                            .offset_for(next_index, self.estimated_item_main, self.item_spacing)
                    });
                    let max_offset =
                        next_offset + self.padding_main - self.axis.main(&placement.size);
                    main_offset = main_offset.min(max_offset);
                }
            }
            let position = self
                .axis
                .position(main_offset, self.padding_cross + cross_offset);
            output.place_child(placement.child_id, position);
        }

        Ok(self.axis.pack_size(reported_main, cross_with_padding))
    }
}

impl LazyListLayout {
    fn is_sticky(&self, index: usize) -> bool {
        self.sticky_indices.binary_search(&index).is_ok()
    }

    fn next_sticky_after(&self, index: usize) -> Option<usize> {
        match self.sticky_indices.binary_search(&index) {
            Ok(pos) => self.sticky_indices.get(pos + 1).copied(),
            Err(pos) => self.sticky_indices.get(pos).copied(),
        }
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
fn compute_visible_children(
    plan: &LazySlotPlan,
    cache: &LazyListCache,
    total_count: usize,
    scroll_offset: Px,
    viewport_span: Px,
    overscan: usize,
    estimated_main: Px,
    spacing: Px,
) -> Vec<VisibleChild> {
    if total_count == 0 {
        return Vec::new();
    }

    let mut start_index = cache.index_for_offset(scroll_offset, estimated_main, spacing);
    let end_target = scroll_offset + viewport_span;
    let mut end_index = cache.index_for_offset(end_target, estimated_main, spacing) + 1;

    start_index = start_index.saturating_sub(overscan);
    end_index = (end_index + overscan).min(total_count);
    if start_index >= end_index {
        end_index = (start_index + 1).min(total_count);
        start_index = start_index.saturating_sub(1);
    }

    let mut result = plan.visible_children(start_index..end_index);
    if let Some(sticky_index) = plan.last_sticky_before(start_index) {
        let existing = result
            .iter()
            .position(|child| child.item_index == sticky_index);
        let sticky_child = existing
            .map(|pos| result.remove(pos))
            .or_else(|| plan.visible_child(sticky_index));
        if let Some(child) = sticky_child {
            result.push(child);
        }
    }
    result
}

fn clamp_reported_main(
    axis: LazyListAxis,
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

fn compute_cross_offset(final_cross: Px, child_cross: Px, alignment: CrossAxisAlignment) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (final_cross - child_cross).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (final_cross - child_cross).max(Px::ZERO),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LazyListAxis {
    Vertical,
    Horizontal,
}

impl LazyListAxis {
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

    fn scroll_offset(&self, position: PxPosition) -> Px {
        match self {
            Self::Vertical => (-position.y).max(Px::ZERO),
            Self::Horizontal => (-position.x).max(Px::ZERO),
        }
    }

    fn child_constraint(&self, parent: ParentConstraint<'_>) -> Constraint {
        match self {
            Self::Vertical => Constraint::new(
                parent.width(),
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
                parent.height(),
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

#[derive(Clone)]
struct Placement {
    item_index: usize,
    child_id: NodeId,
    offset_main: Px,
    size: ComputedData,
}

#[derive(Clone)]
enum LazySlot {
    Items(LazyItemsSlot),
    Sticky(LazyStickySlot),
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

    fn sticky<F>(builder: F, key_hash: Option<u64>) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self::Sticky(LazyStickySlot {
            builder: Arc::new(builder),
            key_hash,
        })
    }

    fn len(&self) -> usize {
        match self {
            Self::Items(slot) => slot.count,
            Self::Sticky(_) => 1,
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
struct LazyStickySlot {
    builder: Arc<dyn Fn() + Send + Sync>,
    key_hash: Option<u64>,
}

#[derive(Clone)]
struct LazySlotPlan {
    entries: Vec<LazySlotEntry>,
    total_count: usize,
    sticky_indices: Vec<usize>,
}

impl LazySlotPlan {
    fn new(slots: Vec<LazySlot>) -> Self {
        let mut entries = Vec::with_capacity(slots.len());
        let mut cursor = 0;
        let mut sticky_indices = Vec::new();
        for slot in slots {
            let len = slot.len();
            if matches!(slot, LazySlot::Sticky(_)) {
                sticky_indices.push(cursor);
            }
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
            sticky_indices,
        }
    }

    fn total_count(&self) -> usize {
        self.total_count
    }

    fn sticky_indices(&self) -> &[usize] {
        &self.sticky_indices
    }

    fn last_sticky_before(&self, index: usize) -> Option<usize> {
        if self.sticky_indices.is_empty() {
            return None;
        }
        match self.sticky_indices.binary_search(&index) {
            Ok(pos) => self.sticky_indices.get(pos).copied(),
            Err(0) => None,
            Err(pos) => self.sticky_indices.get(pos.saturating_sub(1)).copied(),
        }
    }

    fn visible_child(&self, index: usize) -> Option<VisibleChild> {
        let resolved = self.resolve(index)?;
        let key_hash = match resolved {
            ResolvedSlot::Items(slot, local_index) => {
                if let Some(provider) = &slot.key_provider {
                    provider(local_index)
                } else {
                    let mut hasher = DefaultHasher::new();
                    index.hash(&mut hasher);
                    hasher.finish()
                }
            }
            ResolvedSlot::Sticky(slot) => slot.key_hash.unwrap_or_else(|| {
                let mut hasher = DefaultHasher::new();
                index.hash(&mut hasher);
                hasher.finish()
            }),
        };
        let (builder, local_index) = match resolved {
            ResolvedSlot::Items(slot, local_index) => (slot.builder.clone(), local_index),
            ResolvedSlot::Sticky(slot) => {
                let builder = slot.builder.clone();
                (
                    {
                        let wrapper: Arc<dyn Fn(usize) + Send + Sync> = Arc::new(move |_| {
                            builder();
                        });
                        wrapper
                    },
                    0,
                )
            }
        };
        Some(VisibleChild {
            item_index: index,
            local_index,
            builder,
            key_hash,
        })
    }

    fn visible_children(&self, range: Range<usize>) -> Vec<VisibleChild> {
        let mut result = Vec::new();
        for index in range {
            if let Some(child) = self.visible_child(index) {
                result.push(child);
            }
        }
        result
    }

    fn resolve(&self, index: usize) -> Option<ResolvedSlot<'_>> {
        self.entries.iter().find_map(|entry| {
            if index >= entry.start && index < entry.start + entry.len {
                let local_index = index - entry.start;
                match &entry.slot {
                    LazySlot::Items(slot) => Some(ResolvedSlot::Items(slot, local_index)),
                    LazySlot::Sticky(slot) => Some(ResolvedSlot::Sticky(slot)),
                }
            } else {
                None
            }
        })
    }
}

enum ResolvedSlot<'a> {
    Items(&'a LazyItemsSlot, usize),
    Sticky(&'a LazyStickySlot),
}

#[derive(Clone)]
struct LazySlotEntry {
    start: usize,
    len: usize,
    slot: LazySlot,
}

#[derive(Clone)]
struct VisibleChild {
    item_index: usize,
    local_index: usize,
    builder: Arc<dyn Fn(usize) + Send + Sync>,
    key_hash: u64,
}

#[derive(Default)]
struct LazyListCache {
    total_items: usize,
    measured_main: Vec<Option<Px>>,
    fenwick: FenwickTree,
}

impl LazyListCache {
    fn set_item_count(&mut self, count: usize) {
        if self.total_items == count {
            return;
        }
        self.total_items = count;
        self.measured_main = vec![None; count];
        self.fenwick.resize(count);
    }

    fn record_measurement(&mut self, index: usize, actual: Px, estimated: Px) {
        if index >= self.total_items {
            return;
        }
        let previous = self.measured_main[index];
        if previous == Some(actual) {
            return;
        }

        let prev_delta = previous.map(|val| val - estimated).unwrap_or(Px::ZERO);
        let new_delta = actual - estimated;
        let delta_change = new_delta - prev_delta;
        self.measured_main[index] = Some(actual);
        self.fenwick.update(index, delta_change);
    }

    fn offset_for(&self, index: usize, estimated: Px, spacing: Px) -> Px {
        if self.total_items == 0 {
            return Px::ZERO;
        }
        let clamped = index.min(self.total_items);
        let spacing_contrib = px_mul(spacing, clamped);
        let estimated_contrib = px_mul(estimated, clamped);
        spacing_contrib + estimated_contrib + self.fenwick.prefix_sum(clamped)
    }

    fn total_main_size(&self, estimated: Px, spacing: Px) -> Px {
        if self.total_items == 0 {
            return Px::ZERO;
        }
        let spacing_contrib = px_mul(spacing, self.total_items.saturating_sub(1));
        let estimated_contrib = px_mul(estimated, self.total_items);
        spacing_contrib + estimated_contrib + self.fenwick.prefix_sum(self.total_items)
    }

    fn index_for_offset(&self, offset: Px, estimated: Px, spacing: Px) -> usize {
        if self.total_items == 0 {
            return 0;
        }

        let mut low = 0usize;
        let mut high = self.total_items;
        while low < high {
            let mid = (low + high) / 2;
            if self.offset_for(mid, estimated, spacing) <= offset {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low.saturating_sub(1)
            .min(self.total_items.saturating_sub(1))
    }
}

#[derive(Default, Clone)]
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

fn apply_cross_padding(constraint: &mut Constraint, axis: LazyListAxis, padding: Px) {
    let total_padding = padding + padding;
    match axis {
        LazyListAxis::Vertical => {
            constraint.width = shrink_dimension_max(constraint.width, total_padding);
        }
        LazyListAxis::Horizontal => {
            constraint.height = shrink_dimension_max(constraint.height, total_padding);
        }
    }
}

fn shrink_dimension_max(dim: DimensionValue, amount: Px) -> DimensionValue {
    match dim {
        DimensionValue::Fixed(px) => DimensionValue::Fixed(saturating_sub_px(px, amount)),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min,
            max: max.map(|m| saturating_sub_px(m, amount)),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min,
            max: max.map(|m| saturating_sub_px(m, amount)),
        },
    }
}

fn saturating_sub_px(lhs: Px, rhs: Px) -> Px {
    Px(lhs.0.saturating_sub(rhs.0))
}
