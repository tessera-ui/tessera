//! Virtualized list components (`lazy_column` and `lazy_row`) for Tessera UI.
//!
//! These components only instantiate and measure the children that intersect the current
//! viewport, drastically reducing the work needed for long scrolling feeds. They reuse the
//! existing [`scrollable`](crate::scrollable::scrollable) infrastructure for scroll physics
//! and scrollbars, layering a virtualization strategy on top.

use std::{ops::Range, sync::Arc};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, MeasurementError, NodeId, Px, PxPosition, tessera,
};

use crate::{
    alignment::CrossAxisAlignment,
    scrollable::{ScrollableArgs, ScrollableState, scrollable},
};

const DEFAULT_VIEWPORT_ITEMS: usize = 8;

/// Persistent state shared by lazy list components.
#[derive(Default)]
pub struct LazyListState {
    scrollable_state: Arc<ScrollableState>,
    cache: Arc<RwLock<LazyListCache>>,
}

impl LazyListState {
    /// Creates a new lazy list state with default scroll offsets and caches.
    pub fn new() -> Self {
        Self::default()
    }

    fn scrollable_state(&self) -> Arc<ScrollableState> {
        self.scrollable_state.clone()
    }

    fn cache(&self) -> Arc<RwLock<LazyListCache>> {
        self.cache.clone()
    }

    fn override_scroll_extent(&self, axis: LazyListAxis, main: Px, cross: Px) {
        let size = axis.pack_size(main, cross);
        self.scrollable_state.override_child_size(size);
    }
}

/// Arguments shared between lazy lists.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct LazyColumnArgs {
    /// Scroll container arguments. Vertical scrolling is enforced.
    #[builder(default = "ScrollableArgs::default()")]
    pub scrollable: ScrollableArgs,
    /// How children are aligned along the cross axis (horizontal for columns).
    #[builder(default = "CrossAxisAlignment::Start")]
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Gap between successive items.
    #[builder(default = "Dp(0.0)")]
    pub item_spacing: Dp,
    /// Number of extra items instantiated before/after the viewport.
    #[builder(default = "2")]
    pub overscan: usize,
    /// Estimated main-axis size for each item, used before real measurements exist.
    #[builder(default = "Dp(48.0)")]
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the lazy list content.
    #[builder(default = "Dp(0.0)")]
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents. Prevents gigantic textures
    /// when nesting the list inside wrap/auto-sized surfaces.
    #[builder(default = "Some(Px(8192))")]
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyColumnArgs {
    fn default() -> Self {
        LazyColumnArgsBuilder::default().build().unwrap()
    }
}

/// Arguments for `lazy_row`. Identical to [`LazyColumnArgs`] but horizontal scrolling is enforced.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct LazyRowArgs {
    #[builder(default = "ScrollableArgs::default()")]
    pub scrollable: ScrollableArgs,
    #[builder(default = "CrossAxisAlignment::Start")]
    pub cross_axis_alignment: CrossAxisAlignment,
    #[builder(default = "Dp(0.0)")]
    pub item_spacing: Dp,
    #[builder(default = "2")]
    pub overscan: usize,
    #[builder(default = "Dp(48.0)")]
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the lazy list content.
    #[builder(default = "Dp(0.0)")]
    pub content_padding: Dp,
    #[builder(default = "Some(Px(8192))")]
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyRowArgs {
    fn default() -> Self {
        LazyRowArgsBuilder::default().build().unwrap()
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
        self.slots.push(LazySlot::items(count, builder));
    }

    /// Adds lazily generated items from an iterator, providing both index and element reference.
    ///
    /// The iterator is eagerly collected so it can be accessed on demand while virtualizing.
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
        self.slots.push(LazySlot::items(count, {
            let items = items.clone();
            let builder = builder.clone();
            move |idx| {
                if let Some(item) = items.get(idx) {
                    builder(idx, item);
                }
            }
        }));
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

pub type LazyColumnScope<'a> = LazyListScope<'a>;
pub type LazyRowScope<'a> = LazyListScope<'a>;

#[tessera]
pub fn lazy_column<F>(args: LazyColumnArgs, state: Arc<LazyListState>, configure: F)
where
    F: FnOnce(&mut LazyColumnScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyColumnScope { slots: &mut slots };
        configure(&mut scope);
    }

    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = true;
    scrollable_args.horizontal = false;

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

    let state_for_child = state.clone();
    scrollable(scrollable_args, state.scrollable_state(), move || {
        lazy_list_view(view_args, state_for_child.clone(), slots.clone());
    });
}

#[tessera]
pub fn lazy_row<F>(args: LazyRowArgs, state: Arc<LazyListState>, configure: F)
where
    F: FnOnce(&mut LazyRowScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyRowScope { slots: &mut slots };
        configure(&mut scope);
    }

    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = false;
    scrollable_args.horizontal = true;

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

    let state_for_child = state.clone();
    scrollable(scrollable_args, state.scrollable_state(), move || {
        lazy_list_view(view_args, state_for_child.clone(), slots.clone());
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
fn lazy_list_view(view_args: LazyListViewArgs, state: Arc<LazyListState>, slots: Vec<LazySlot>) {
    let plan = LazySlotPlan::new(slots);
    let total_count = plan.total_count();

    let cache = state.cache();
    {
        let mut guard = cache.write();
        guard.set_item_count(total_count);
    }

    let scroll_offset = view_args
        .axis
        .scroll_offset(state.scrollable_state().child_position());
    let padding_main = view_args.padding_main;
    let viewport_span = resolve_viewport_span(
        view_args
            .axis
            .visible_span(state.scrollable_state().visible_size()),
        view_args.estimated_item_main,
        view_args.item_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);

    let visible_children = {
        let cache_guard = cache.read();
        compute_visible_children(
            &plan,
            &cache_guard,
            total_count,
            scroll_offset,
            viewport_span,
            view_args.overscan,
            view_args.estimated_item_main,
            view_args.item_spacing,
        )
    };

    if visible_children.is_empty() {
        measure(Box::new(move |_| Ok(ComputedData::ZERO)));
        return;
    }

    let cache_for_measure = cache.clone();
    let viewport_limit = viewport_span + padding_main + padding_main;
    let state_for_measure = state.clone();
    let child_constraint_axis = view_args.axis;
    let estimated_item_main = view_args.estimated_item_main;
    let spacing = view_args.item_spacing;
    let cross_alignment = view_args.cross_axis_alignment;
    let padding_cross = view_args.padding_cross;
    let visible_plan = visible_children.clone();

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            if input.children_ids.len() != visible_plan.len() {
                return Err(MeasurementError::MeasureFnFailed(
                    "Lazy list measured child count mismatch".into(),
                ));
            }

            let mut child_constraint =
                child_constraint_axis.child_constraint(input.parent_constraint);
            apply_cross_padding(&mut child_constraint, child_constraint_axis, padding_cross);
            let mut placements = Vec::with_capacity(visible_plan.len());
            let mut max_cross = Px::ZERO;
            {
                let mut cache_guard = cache_for_measure.write();

                for (visible, child_id) in visible_plan.iter().zip(input.children_ids.iter()) {
                    let item_offset =
                        cache_guard.offset_for(visible.item_index, estimated_item_main, spacing);
                    let child_size = input.measure_child(*child_id, &child_constraint)?;

                    cache_guard.record_measurement(
                        visible.item_index,
                        child_constraint_axis.main(&child_size),
                        estimated_item_main,
                    );

                    max_cross = max_cross.max(child_constraint_axis.cross(&child_size));
                    placements.push(Placement {
                        child_id: *child_id,
                        offset_main: item_offset,
                        size: child_size,
                    });
                }
            }

            let total_main = cache_for_measure
                .read()
                .total_main_size(estimated_item_main, spacing);

            let inner_cross = max_cross;
            let total_main_with_padding = total_main + padding_main + padding_main;
            let cross_with_padding = inner_cross + padding_cross + padding_cross;
            state_for_measure.override_scroll_extent(
                child_constraint_axis,
                total_main_with_padding,
                cross_with_padding,
            );

            let reported_main = clamp_reported_main(
                child_constraint_axis,
                input.parent_constraint,
                total_main_with_padding,
                viewport_limit,
                view_args.max_viewport_main,
            );

            for placement in &placements {
                let cross_offset = compute_cross_offset(
                    inner_cross,
                    child_constraint_axis.cross(&placement.size),
                    cross_alignment,
                );
                let position = child_constraint_axis.position(
                    placement.offset_main + padding_main,
                    padding_cross + cross_offset,
                );
                input.place_child(placement.child_id, position);
            }

            Ok(child_constraint_axis.pack_size(reported_main, cross_with_padding))
        },
    ));

    for child in build_child_closures(&visible_children) {
        child();
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

    plan.visible_children(start_index..end_index)
}

fn clamp_reported_main(
    axis: LazyListAxis,
    parent_constraint: &Constraint,
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

#[derive(Clone, Copy)]
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

    fn child_constraint(&self, parent: &Constraint) -> Constraint {
        match self {
            Self::Vertical => Constraint::new(
                parent.width,
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
                parent.height,
            ),
        }
    }

    fn constraint_max(&self, constraint: &Constraint) -> Option<Px> {
        match self {
            Self::Vertical => constraint.height.get_max(),
            Self::Horizontal => constraint.width.get_max(),
        }
    }
}

#[derive(Clone)]
struct Placement {
    child_id: NodeId,
    offset_main: Px,
    size: ComputedData,
}

#[derive(Clone)]
enum LazySlot {
    Items(LazyItemsSlot),
}

impl LazySlot {
    fn items<F>(count: usize, builder: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        Self::Items(LazyItemsSlot {
            count,
            builder: Arc::new(builder),
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

    fn visible_children(&self, range: Range<usize>) -> Vec<VisibleChild> {
        let mut result = Vec::new();
        for index in range {
            if let Some((slot, local_index)) = self.resolve(index) {
                result.push(VisibleChild {
                    item_index: index,
                    local_index,
                    builder: slot.builder.clone(),
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
struct VisibleChild {
    item_index: usize,
    local_index: usize,
    builder: Arc<dyn Fn(usize) + Send + Sync>,
}

fn build_child_closures(children: &[VisibleChild]) -> Vec<Box<dyn FnOnce()>> {
    children
        .iter()
        .map(|child| {
            let builder = child.builder.clone();
            let local_index = child.local_index;
            Box::new(move || (builder)(local_index)) as Box<dyn FnOnce()>
        })
        .collect()
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
