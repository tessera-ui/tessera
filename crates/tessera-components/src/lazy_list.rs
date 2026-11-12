//! Virtualized list components for displaying long scrolling feeds.
//!
//! ## Usage
//!
//! Use `lazy_column` or `lazy_row` to efficiently display large datasets.
use std::{collections::BTreeMap, hash::Hash, ops::Range, sync::Arc};

use tessera_ui::{
    AxisConstraint, CallbackWith, Constraint, Dp, FocusScopeNode, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlotWith, State, key,
    layout::{MeasureScope, PlacementResult, PlacementScope, Stack, layout},
    modifier::FocusModifierExt as _,
    remember, remember_with_key, tessera,
};

use crate::{
    alignment::CrossAxisAlignment,
    layout_flex::px_mul,
    lazy_shared::{
        FenwickTree, LazyAxis, LazyItemsSlot, MeasuredMain, ZeroLayout, clamp_reported_main,
        hash_key,
    },
    scrollable::{ScrollBarBehavior, ScrollableController, scrollable, synchronize_scroll_density},
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
/// use tessera_components::lazy_list::{LazyListController, lazy_column};
/// use tessera_ui::{LayoutResult, retain_with_key, tessera};
///
/// #[tessera]
/// fn scrollable_page(page_id: String) {
///     // Both scroll position and measurement cache persist across navigation
///     let controller = retain_with_key(page_id.clone(), LazyListController::new);
///     lazy_column().controller(controller).items(100, |i| {
///         let _ = i;
///     });
/// }
/// ```
pub struct LazyListController {
    scroll: ScrollableController,
    cache: LazyListCache,
    pending_position: Option<PxPosition>,
    pending_item: Option<ItemScrollRequest>,
    viewport: Option<State<ScrollableController>>,
    geometry: Option<ListScrollGeometry>,
    composed: BTreeMap<usize, u64>,
    focused_item: Option<usize>,
    retains_composition: bool,
}

struct ListAttachment {
    controller: State<LazyListController>,
    viewport: State<ScrollableController>,
}

impl Drop for ListAttachment {
    fn drop(&mut self) {
        if self.controller.is_alive()
            && self
                .controller
                .with(|controller| controller.viewport == Some(self.viewport))
        {
            self.controller.with_mut(|controller| {
                controller.composed.clear();
                controller.focused_item = None;
                controller.retains_composition = false;
                controller.viewport = None;
                controller.geometry = None;
            });
        }
    }
}

#[derive(Clone, Copy)]
struct ItemScrollRequest {
    index: usize,
    offset: Dp,
    position: Option<PxPosition>,
}

#[derive(Clone, Copy, PartialEq)]
struct ListScrollGeometry {
    axis: LazyAxis,
    estimated_main: Px,
    spacing: Px,
    padding: Px,
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
            pending_position: None,
            pending_item: None,
            viewport: None,
            geometry: None,
            composed: BTreeMap::new(),
            focused_item: None,
            retains_composition: false,
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

    /// Scrolls to content offsets from the start of each axis, in logical
    /// pixels.
    ///
    /// The request is applied once when the list next builds. Subsequent wheel
    /// and drag input continues from that position.
    pub fn scroll_to(&mut self, x: Dp, y: Dp) {
        let position = PxPosition::new(-Dp(x.0.max(0.0)).to_px(), -Dp(y.0.max(0.0)).to_px());
        self.scroll.set_scroll_position(position);
        self.pending_position = Some(position);
        self.pending_item = None;
    }

    /// Reveals an item at an offset from the viewport start.
    ///
    /// Earlier unmeasured items use the list estimate. The requested item is
    /// composed and measured before its final visible placement is published.
    /// Requests outside the current item range are ignored. Later wheel input
    /// continues from the request rather than being overwritten by it.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is not finite.
    pub fn scroll_to_item(&mut self, index: usize, offset: Dp) {
        assert!(offset.0.is_finite(), "item offset must be finite");
        let mut request = ItemScrollRequest {
            index,
            offset,
            position: None,
        };
        if let Some(geometry) = self.geometry {
            if index >= self.cache.total_items {
                return;
            }
            let main = (self
                .cache
                .offset_for(index, geometry.estimated_main, geometry.spacing)
                + geometry.padding
                - Px::from(offset))
            .max(Px::ZERO);
            let position = geometry.axis.scroll_position(main);
            self.scroll.set_scroll_position(position);
            if let Some(viewport) = self.viewport.filter(State::is_alive) {
                viewport.with_mut(|viewport| viewport.set_scroll_position(position));
            }
            request.position = Some(position);
        }
        self.pending_position = None;
        self.pending_item = Some(request);
    }
}

fn restore_scroll_position(
    controller: State<LazyListController>,
    viewport: State<ScrollableController>,
) {
    let density = tessera_ui::dp::scale_factor();
    if controller.with(|controller| controller.cache.density != density) {
        controller.with_mut(|controller| {
            let ratio = controller.scroll.update_density(density);
            if let Some(position) = &mut controller.pending_position {
                position.x = (f64::from(position.x) * ratio) as f32;
                position.y = (f64::from(position.y) * ratio) as f32;
            }
            if let Some(position) = controller
                .pending_item
                .as_mut()
                .and_then(|request| request.position.as_mut())
            {
                position.x = (f64::from(position.x) * ratio) as f32;
                position.y = (f64::from(position.y) * ratio) as f32;
            }
        });
    }
    synchronize_scroll_density(viewport);
    let request = controller.with(|controller| controller.pending_position);
    if request.is_some() {
        controller.with_mut(|controller| controller.pending_position = None);
    }
    let saved = controller.with(|controller| controller.scroll.child_position());
    let restore = saved != PxPosition::ZERO && !viewport.with(|viewport| viewport.is_positioned());
    if let Some(position) = request.or_else(|| restore.then_some(saved)) {
        viewport.with_mut(|viewport| viewport.set_scroll_position(position));
    }
}

/// Accumulated lazy-list item declarations carried by the builder.
#[derive(Clone, Default, PartialEq)]
pub struct LazyListContent {
    slots: Vec<LazySlot>,
}

impl LazyListContent {
    fn push(&mut self, slot: LazySlot) {
        self.slots.push(slot);
    }
}

/// # lazy_column
///
/// A vertically scrolling list that only renders items visible in the viewport.
///
/// ## Usage
///
/// Display a long, vertical list of items without incurring the performance
/// cost of rendering every item at once.
/// Remeasurement preserves the first item start visible in the viewport. If no
/// item start is visible, the partially visible item retains its offset.
///
/// ## Parameters
///
/// - `modifier` — optional modifier for the scroll container.
/// - `cross_axis_alignment` — how items align along the cross axis.
/// - `item_spacing` — gap between successive items.
/// - `overscan` — number of extra items instantiated before and after the
///   viewport.
/// - `estimated_item_size` — estimated main-axis size used before real
///   measurements exist.
/// - `content_padding` — symmetric padding around the lazy list content.
/// - `max_viewport_main` — optional maximum viewport length, also limiting
///   initial item construction before the viewport is measured.
/// - `scroll_smoothing` — fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
/// - `scrollbar_behavior` — scrollbar visibility; visible by default.
/// - `controller` — optional external controller for scroll position and cache.
/// - `viewport_controller` — optional shared controller for the actual scroll
///   viewport.
/// - `keep_composed_items` — retain visited item state until this list
///   unmounts; hidden items are unplaced, except the active focused item kept
///   for reveal.
/// - item declarations are appended through builder methods like `item`,
///   `items`, and `sticky_header`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{lazy_list::lazy_column, text::text};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     lazy_column().items(1000, |i| {
///         let text_content = format!("Item #{i}");
///         text(text_content);
///     });
/// }
/// ```
#[tessera]
pub fn lazy_column(
    modifier: Option<Modifier>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    item_spacing: Option<Dp>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    controller: Option<State<LazyListController>>,
    viewport_controller: Option<State<ScrollableController>>,
    keep_composed_items: Option<bool>,
    #[prop(skip_setter)] content: Option<LazyListContent>,
) {
    let content = content.unwrap_or_default();
    let controller = match controller {
        Some(value) => value,
        None => remember(LazyListController::new),
    };
    let scroll_controller = if let Some(controller) = viewport_controller {
        controller
    } else {
        remember(ScrollableController::default)
    };
    let args = LazyListSlotsArgs {
        modifier: modifier.unwrap_or_default(),
        cross_axis_alignment: cross_axis_alignment.unwrap_or_default(),
        item_spacing: item_spacing.unwrap_or(Dp(0.0)),
        overscan: overscan.unwrap_or(0),
        estimated_item_size: estimated_item_size.unwrap_or(Dp(0.0)),
        content_padding: content_padding.unwrap_or(Dp(0.0)),
        max_viewport_main,
        scroll_smoothing: scroll_smoothing.unwrap_or(crate::scrollable::DEFAULT_SCROLL_SMOOTHING),
        scrollbar_behavior: scrollbar_behavior.unwrap_or_default(),
        controller,
        scroll_controller,
        keep_composed_items: keep_composed_items.unwrap_or(false),
        slots: content.slots,
    };
    let parameters = remember(|| args.clone());
    if parameters.with(|parameters| parameters != &args) {
        parameters.set(args);
    }
    lazy_column_slots(parameters);
}

#[derive(Clone, PartialEq)]
struct LazyListSlotsArgs {
    modifier: Modifier,
    cross_axis_alignment: CrossAxisAlignment,
    item_spacing: Dp,
    overscan: usize,
    estimated_item_size: Dp,
    content_padding: Dp,
    max_viewport_main: Option<Px>,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    controller: State<LazyListController>,
    scroll_controller: State<ScrollableController>,
    keep_composed_items: bool,
    slots: Vec<LazySlot>,
}

#[tessera]
fn lazy_column_slots(parameters: State<LazyListSlotsArgs>) {
    let args = parameters.get();
    // The viewport controller is recreated whenever the list is remounted,
    // while the list controller retains the last scroll position. Restore
    // that position once, before the viewport has been positioned;
    // comparing every frame would overwrite a newer wheel or drag offset
    // with the previous frame's value.
    let scroll_controller = args.scroll_controller;
    restore_scroll_position(args.controller, scroll_controller);

    scrollable()
        .modifier(args.modifier)
        .vertical(true)
        .horizontal(false)
        .apply_child_offset(false)
        .scroll_smoothing(args.scroll_smoothing)
        .scrollbar_behavior(args.scrollbar_behavior)
        .controller(scroll_controller)
        .child(move || {
            let args = parameters.get();
            let scroll_controller = args.scroll_controller;
            let item_spacing = Px::from(args.item_spacing).max(Px::ZERO);
            let estimated_item_main = ensure_positive_px(Px::from(args.estimated_item_size));
            let padding_main = Px::from(args.content_padding).max(Px::ZERO);
            let padding_cross = Px::from(args.content_padding).max(Px::ZERO);
            // Sync scroll position back to controller
            let current_pos = scroll_controller.with(|sc| sc.child_position());
            let should_sync_position = args
                .controller
                .with(|c| c.scroll.child_position() != current_pos);
            if should_sync_position {
                args.controller
                    .with_mut(|c| c.scroll.set_scroll_position(current_pos));
            }
            lazy_list_view()
                .axis(LazyAxis::Vertical)
                .cross_axis_alignment(args.cross_axis_alignment)
                .item_spacing(item_spacing)
                .estimated_item_main(estimated_item_main)
                .overscan(args.overscan)
                .padding_main(padding_main)
                .padding_cross(padding_cross)
                .slots(args.slots.clone())
                .controller(args.controller)
                .scroll_controller(scroll_controller)
                .keep_composed_items(args.keep_composed_items)
                .max_viewport_main_optional(args.max_viewport_main);
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
/// - `modifier` — optional modifier for the scroll container.
/// - `cross_axis_alignment` — how items align along the cross axis.
/// - `item_spacing` — gap between successive items.
/// - `overscan` — number of extra items instantiated before and after the
///   viewport.
/// - `estimated_item_size` — estimated main-axis size used before real
///   measurements exist.
/// - `content_padding` — symmetric padding around the lazy list content.
/// - `max_viewport_main` — optional maximum viewport length, also limiting
///   initial item construction before the viewport is measured.
/// - `scroll_smoothing` — fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
/// - `scrollbar_behavior` — scrollbar visibility; visible by default.
/// - `controller` — optional external controller for scroll position and cache.
/// - `viewport_controller` — optional shared controller for the actual scroll
///   viewport.
/// - `keep_composed_items` — retain visited item state until this list
///   unmounts; hidden items are unplaced, except the active focused item kept
///   for reveal.
/// - item declarations are appended through builder methods like `item`,
///   `items`, and `sticky_header`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{lazy_list::lazy_row, text::text};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     lazy_row().items(100, |i| {
///         let text_content = format!("Item {i}");
///         text(text_content);
///     });
/// }
/// ```
#[tessera]
pub fn lazy_row(
    modifier: Option<Modifier>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    item_spacing: Option<Dp>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    controller: Option<State<LazyListController>>,
    viewport_controller: Option<State<ScrollableController>>,
    keep_composed_items: Option<bool>,
    #[prop(skip_setter)] content: Option<LazyListContent>,
) {
    let content = content.unwrap_or_default();
    let controller = match controller {
        Some(value) => value,
        None => remember(LazyListController::new),
    };
    let scroll_controller = if let Some(controller) = viewport_controller {
        controller
    } else {
        remember(ScrollableController::default)
    };
    let args = LazyListSlotsArgs {
        modifier: modifier.unwrap_or_default(),
        cross_axis_alignment: cross_axis_alignment.unwrap_or_default(),
        item_spacing: item_spacing.unwrap_or(Dp(0.0)),
        overscan: overscan.unwrap_or(0),
        estimated_item_size: estimated_item_size.unwrap_or(Dp(0.0)),
        content_padding: content_padding.unwrap_or(Dp(0.0)),
        max_viewport_main,
        scroll_smoothing: scroll_smoothing.unwrap_or(crate::scrollable::DEFAULT_SCROLL_SMOOTHING),
        scrollbar_behavior: scrollbar_behavior.unwrap_or_default(),
        controller,
        scroll_controller,
        keep_composed_items: keep_composed_items.unwrap_or(false),
        slots: content.slots,
    };
    let parameters = remember(|| args.clone());
    if parameters.with(|parameters| parameters != &args) {
        parameters.set(args);
    }
    lazy_row_slots(parameters);
}

macro_rules! impl_lazy_list_builder {
    ($builder:ident) => {
        impl $builder {
            /// Append indexed items with stable keys and a shared render slot.
            pub fn items_with_key_shared<K, KF>(
                &self,
                count: usize,
                key_provider: KF,
                builder: impl Into<RenderSlotWith<usize>> + 'static,
            ) -> Self
            where
                K: Hash,
                KF: Fn(usize) -> K + 'static,
            {
                self.props.append("content", move |props| {
                    let slot = {
                        let builder = builder.into();
                        LazySlot::Items(LazyItemsSlot {
                            count,
                            builder,
                            key_provider: Some(CallbackWith::new(move |index| {
                                hash_key(key_provider(index))
                            })),
                        })
                    };
                    props
                        .content
                        .get_or_insert_with(LazyListContent::default)
                        .push(slot);
                });
                Self { props: self.props }
            }

            /// Append a sticky header item.
            pub fn sticky_header<F, Output>(&self, builder: F) -> Self
            where
                F: Fn() -> Output + 'static,
            {
                self.props.append("content", move |props| {
                    let slot = LazySlot::sticky(
                        RenderSlotWith::new(move |_| {
                            builder();
                        }),
                        None,
                    );
                    props
                        .content
                        .get_or_insert_with(LazyListContent::default)
                        .push(slot);
                });
                Self { props: self.props }
            }

            /// Append a sticky header item with a stable key.
            pub fn sticky_header_with_key<K, F, Output>(&self, key: K, builder: F) -> Self
            where
                K: Hash,
                F: Fn() -> Output + 'static,
            {
                let key = hash_key(key);
                self.props.append("content", move |props| {
                    let slot = LazySlot::sticky(
                        RenderSlotWith::new(move |_| {
                            builder();
                        }),
                        Some(key),
                    );
                    props
                        .content
                        .get_or_insert_with(LazyListContent::default)
                        .push(slot);
                });
                Self { props: self.props }
            }

            /// Append a sticky header with a stable key and a shared render slot.
            pub fn sticky_header_with_key_shared<K>(
                &self,
                key: K,
                builder: impl Into<RenderSlotWith<usize>> + 'static,
            ) -> Self
            where
                K: Hash,
            {
                let key_hash = Some(hash_key(key));
                self.props.append("content", move |props| {
                    let slot = {
                        let builder = builder.into();
                        LazySlot::Sticky(LazyStickySlot { builder, key_hash })
                    };
                    props
                        .content
                        .get_or_insert_with(LazyListContent::default)
                        .push(slot);
                });
                Self { props: self.props }
            }
        }
    };
}

impl_lazy_list_builder!(LazyColumnBuilder);
impl_lazy_list_builder!(LazyRowBuilder);
crate::lazy_shared::impl_lazy_builder!(LazyColumnBuilder, LazyListContent);
crate::lazy_shared::impl_lazy_builder!(LazyRowBuilder, LazyListContent);

#[tessera]
fn lazy_row_slots(parameters: State<LazyListSlotsArgs>) {
    let args = parameters.get();
    // The viewport controller is recreated whenever the list is remounted,
    // while the list controller retains the last scroll position. Restore
    // that position once, before the viewport has been positioned;
    // comparing every frame would overwrite a newer wheel or drag offset
    // with the previous frame's value.
    let scroll_controller = args.scroll_controller;
    restore_scroll_position(args.controller, scroll_controller);

    scrollable()
        .modifier(args.modifier)
        .vertical(false)
        .horizontal(true)
        .apply_child_offset(false)
        .scroll_smoothing(args.scroll_smoothing)
        .scrollbar_behavior(args.scrollbar_behavior)
        .controller(scroll_controller)
        .child(move || {
            let args = parameters.get();
            let scroll_controller = args.scroll_controller;
            let item_spacing = Px::from(args.item_spacing).max(Px::ZERO);
            let estimated_item_main = ensure_positive_px(Px::from(args.estimated_item_size));
            let padding_main = Px::from(args.content_padding).max(Px::ZERO);
            let padding_cross = Px::from(args.content_padding).max(Px::ZERO);
            // Sync scroll position back to controller
            let current_pos = scroll_controller.with(|sc| sc.child_position());
            let should_sync_position = args
                .controller
                .with(|c| c.scroll.child_position() != current_pos);
            if should_sync_position {
                args.controller
                    .with_mut(|c| c.scroll.set_scroll_position(current_pos));
            }
            lazy_list_view()
                .axis(LazyAxis::Horizontal)
                .cross_axis_alignment(args.cross_axis_alignment)
                .item_spacing(item_spacing)
                .estimated_item_main(estimated_item_main)
                .overscan(args.overscan)
                .padding_main(padding_main)
                .padding_cross(padding_cross)
                .slots(args.slots.clone())
                .controller(args.controller)
                .scroll_controller(scroll_controller)
                .keep_composed_items(args.keep_composed_items)
                .max_viewport_main_optional(args.max_viewport_main);
        });
}

#[tessera]
fn lazy_list_view(
    axis: Option<LazyAxis>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    item_spacing: Option<Px>,
    estimated_item_main: Option<Px>,
    overscan: Option<usize>,
    max_viewport_main: Option<Px>,
    padding_main: Option<Px>,
    padding_cross: Option<Px>,
    controller: Option<State<LazyListController>>,
    slots: Option<Vec<LazySlot>>,
    scroll_controller: Option<State<ScrollableController>>,
    keep_composed_items: Option<bool>,
) {
    let keep_composed_items = keep_composed_items.unwrap_or(false);
    let axis = axis.unwrap_or_default();
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let item_spacing = item_spacing.unwrap_or(Px::ZERO);
    let estimated_item_main = estimated_item_main.unwrap_or(Px::ZERO);
    let overscan = overscan.unwrap_or(0);
    let padding_main = padding_main.unwrap_or(Px::ZERO);
    let padding_cross = padding_cross.unwrap_or(Px::ZERO);
    let slots = slots.unwrap_or_default();
    let controller = controller.expect("lazy_list_view requires controller");
    let scroll_controller = scroll_controller.expect("lazy_list_view requires scroll_controller");
    let _attachment = remember_with_key((controller, scroll_controller), || ListAttachment {
        controller,
        viewport: scroll_controller,
    });
    let plan = LazySlotPlan::new(slots.clone());
    let total_count = plan.total_count();

    let cache_config_changed = controller.with(|c| {
        c.cache.total_items != total_count
            || c.cache.estimated_main != estimated_item_main
            || c.cache.density != tessera_ui::dp::scale_factor()
    });
    if cache_config_changed {
        controller.with_mut(|c| c.cache.configure(total_count, estimated_item_main));
    }

    let geometry = ListScrollGeometry {
        axis,
        estimated_main: estimated_item_main,
        spacing: item_spacing,
        padding: padding_main,
    };
    if controller.with(|controller| {
        controller.viewport != Some(scroll_controller)
            || controller.geometry != Some(geometry)
            || controller.retains_composition != keep_composed_items
    }) {
        controller.with_mut(|controller| {
            controller.viewport = Some(scroll_controller);
            controller.geometry = Some(geometry);
            controller.retains_composition = keep_composed_items;
        });
    }
    if !keep_composed_items
        && controller
            .with(|controller| !controller.composed.is_empty() || controller.focused_item.is_some())
    {
        controller.with_mut(|controller| {
            controller.composed.clear();
            controller.focused_item = None;
        });
    }
    let request = controller.with(|controller| controller.pending_item);
    if let Some(mut request) = request {
        if request.index >= total_count {
            controller.with_mut(|controller| controller.pending_item = None);
        } else {
            let main = controller.with(|controller| {
                controller
                    .cache
                    .offset_for(request.index, estimated_item_main, item_spacing)
            }) + padding_main
                - Px::from(request.offset);
            let desired = axis.scroll_position(main.max(Px::ZERO));
            if let Some(previous) = request.position {
                let delta = desired - previous;
                if delta != PxPosition::ZERO {
                    scroll_controller.with_mut(|scroll| scroll.translate_scroll_position(delta));
                }
            } else {
                scroll_controller.with_mut(|scroll| scroll.set_scroll_position(desired));
            }
            request.position = Some(desired);
            controller.with_mut(|controller| controller.pending_item = Some(request));
        }
    }
    let scroll_offset = axis.scroll_offset(scroll_controller.with(|s| s.child_position()));
    let viewport_span = resolve_viewport_span(
        axis.main(&scroll_controller.with(|s| s.visible_size())),
        estimated_item_main,
        item_spacing,
    );
    let viewport_span = max_viewport_main
        .map(|limit| viewport_span.min(limit.max(Px::ZERO)))
        .unwrap_or(viewport_span);
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);
    let total_main =
        controller.with(|c| c.cache.total_main_size(estimated_item_main, item_spacing));
    let total_main_with_padding = total_main + padding_main + padding_main;
    let visible_cross = axis.cross(&scroll_controller.with(|s| s.visible_size()));
    // Only the main axis affects lazy-list scroll extents. Cross-axis size is
    // finalized during measurement (with real child sizes), so keep the
    // existing cross value here to avoid view/measure override oscillation.
    let current_child_cross = axis.cross(&scroll_controller.with(|s| s.child_size()));
    let estimated_cross = if current_child_cross > Px::ZERO {
        current_child_cross
    } else {
        visible_cross
    };
    let scroll_child_size = axis.pack_size(total_main_with_padding, estimated_cross);
    let needs_scroll_child_size_update =
        scroll_controller.with(|c| c.child_size() != scroll_child_size);
    if needs_scroll_child_size_update {
        scroll_controller.with_mut(|c| c.override_child_size(scroll_child_size));
    }

    let mut visible_children = controller.with(|c| {
        compute_visible_children(
            &plan,
            &c.cache,
            total_count,
            scroll_offset,
            viewport_span,
            overscan,
            estimated_item_main,
            item_spacing,
        )
    });

    let invalid_focus = controller.with(|controller| {
        controller.focused_item.is_some_and(|index| {
            plan.visible_child(index)
                .is_none_or(|child| controller.composed.get(&index) != Some(&child.key_hash))
        })
    });
    if invalid_focus {
        controller.with_mut(|controller| controller.focused_item = None);
    }
    for index in controller
        .with(|controller| {
            [
                controller.pending_item.map(|request| request.index),
                controller.focused_item,
            ]
        })
        .into_iter()
        .flatten()
    {
        if !visible_children
            .children
            .iter()
            .any(|child| child.item_index == index)
            && let Some(child) = plan.visible_child(index)
        {
            visible_children.children.push(child);
        }
    }
    visible_children
        .children
        .sort_unstable_by_key(|child| child.item_index);
    if visible_children.children.is_empty() {
        if controller
            .with(|controller| !controller.composed.is_empty() || controller.focused_item.is_some())
        {
            controller.with_mut(|controller| {
                controller.composed.clear();
                controller.focused_item = None;
            });
        }
        tessera_ui::layout::layout(ZeroLayout);
        return;
    }

    let next_focus = LazyListFocusArgs {
        axis,
        controller,
        scroll_controller,
        total_count,
        total_main,
        viewport_span,
        scroll_offset,
        visible_range: visible_children.range.clone(),
        estimated_item_main,
        item_spacing,
    };
    let focus_args = remember(|| next_focus.clone());
    if focus_args.with(|args| args != &next_focus) {
        focus_args.set(next_focus);
    }
    let focus_modifier = lazy_list_focus_beyond_bounds_modifier(focus_args);

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_item_indices = visible_children
        .children
        .iter()
        .map(|visible| visible.item_index)
        .collect();

    let mut children = if keep_composed_items {
        let mut history = controller.with(|controller| controller.composed.clone());
        history.retain(|index, key| {
            plan.visible_child(*index)
                .is_some_and(|child| child.key_hash == *key)
        });
        for child in &visible_children.children {
            history.insert(child.item_index, child.key_hash);
        }
        if controller.with(|controller| controller.composed != history) {
            controller.with_mut(|controller| controller.composed = history.clone());
        }
        history
            .keys()
            .filter_map(|index| plan.visible_child(*index))
            .collect::<Vec<_>>()
    } else {
        visible_children.children
    };
    // Sticky items overlap scrolling content and must own the upper paint and
    // hit-test order.
    children.sort_by_key(|child| {
        plan.sticky_indices()
            .binary_search(&child.item_index)
            .is_ok()
    });
    let composed_item_indices = children.iter().map(|child| child.item_index).collect();
    tessera_ui::layout::layout(LazyListLayout {
        axis,
        cross_axis_alignment,
        item_spacing,
        estimated_item_main,
        max_viewport_main,
        padding_main,
        padding_cross,
        viewport_limit,
        visible_item_indices,
        composed_item_indices,
        sticky_indices: plan.sticky_indices().to_vec(),
        scroll_offset,
        controller,
        scroll_controller,
    })
    .modifier(focus_modifier)
    .child(move || {
        for child in &children {
            let child = child.clone();
            key(child.key_hash, || {
                retained_lazy_item(
                    child.item_index,
                    child.local_index,
                    child.builder,
                    controller,
                    keep_composed_items,
                );
            });
        }
    });
}

#[derive(Clone, PartialEq)]
struct LazyListLayout {
    axis: LazyAxis,
    cross_axis_alignment: CrossAxisAlignment,
    item_spacing: Px,
    estimated_item_main: Px,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
    viewport_limit: Px,
    visible_item_indices: Vec<usize>,
    composed_item_indices: Vec<usize>,
    sticky_indices: Vec<usize>,
    scroll_offset: Px,
    controller: State<LazyListController>,
    scroll_controller: State<ScrollableController>,
}

impl tessera_ui::LayoutPolicy for LazyListLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != self.composed_item_indices.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy list measured child count mismatch".into(),
            ));
        }

        let mut measured_children: Vec<_> = self
            .composed_item_indices
            .iter()
            .copied()
            .zip(children.iter().copied())
            .filter(|(index, _)| self.visible_item_indices.binary_search(index).is_ok())
            .collect();
        measured_children.sort_unstable_by_key(|(item_index, _)| *item_index);

        let mut child_constraint = self.axis.child_constraint(
            self.axis.cross_constraint(input.parent_constraint()),
            self.cross_axis_alignment,
        );
        apply_cross_padding(&mut child_constraint, self.axis, self.padding_cross);
        let mut measured_entries = Vec::with_capacity(self.visible_item_indices.len());
        let mut inner_cross = Px::ZERO;
        for (item_index, child) in &measured_children {
            let child_size = child.measure(input, &child_constraint)?;
            let child_size = child_size.size();
            let measured_main = self.axis.main(&child_size);
            inner_cross = inner_cross.max(self.axis.cross(&child_size));
            measured_entries.push((*item_index, *child, child_size, measured_main));
        }

        let cross_limit = match self.axis {
            LazyAxis::Vertical => child_constraint.width.resolve_max(),
            LazyAxis::Horizontal => child_constraint.height.resolve_max(),
        };
        // Collapsed viewports can wrap text into misleading main-axis sizes.
        let has_cache_updates = cross_limit.is_none_or(|size| size > Px::ZERO)
            && self.controller.with(|c| {
                measured_entries
                    .iter()
                    .any(|(item_index, _, _, measured_main)| {
                        c.cache.measurement(*item_index) != Some(*measured_main)
                    })
            });
        if has_cache_updates {
            let scroll_offset = self.axis.scroll_offset(
                self.scroll_controller
                    .with(|controller| controller.child_position()),
            );
            let anchor_delta = self.controller.with_mut(|c| {
                let mut anchor_index =
                    c.pending_item
                        .map(|request| request.index)
                        .unwrap_or_else(|| {
                            c.cache.index_for_offset(
                                (scroll_offset - self.padding_main).max(Px::ZERO),
                                self.estimated_item_main,
                                self.item_spacing,
                            )
                        });
                let first_start =
                    c.cache
                        .offset_for(anchor_index, self.estimated_item_main, self.item_spacing)
                        + self.padding_main;
                let next_start = c.cache.offset_for(
                    anchor_index + 1,
                    self.estimated_item_main,
                    self.item_spacing,
                ) + self.padding_main;
                // A mostly hidden large item must not displace the next visible
                // item start.
                if c.pending_item.is_none()
                    && first_start < scroll_offset
                    && anchor_index + 1 < c.cache.total_items
                    && next_start < scroll_offset + self.viewport_limit
                {
                    anchor_index += 1;
                }
                let anchor_offset = (c.pending_item.is_some()
                    || c.cache.measurement(anchor_index).is_some())
                .then(|| {
                    c.cache
                        .offset_for(anchor_index, self.estimated_item_main, self.item_spacing)
                });
                for (item_index, _, _, measured_main) in &measured_entries {
                    c.cache.record_measurement(
                        *item_index,
                        *measured_main,
                        self.estimated_item_main,
                    );
                }
                anchor_offset.map_or(Px::ZERO, |previous| {
                    c.cache
                        .offset_for(anchor_index, self.estimated_item_main, self.item_spacing)
                        - previous
                })
            });
            if anchor_delta != Px::ZERO {
                self.scroll_controller.with_mut(|controller| {
                    controller.translate_scroll_position(self.axis.scroll_position(anchor_delta));
                });
            }
        }

        if self.controller.with(|controller| {
            controller.pending_item.is_some_and(|request| {
                measured_entries
                    .iter()
                    .any(|(index, _, _, _)| *index == request.index)
            })
        }) {
            self.controller
                .with_mut(|controller| controller.pending_item = None);
        }
        let scroll_offset = self.axis.scroll_offset(
            self.scroll_controller
                .with(|controller| controller.child_position()),
        );

        let (item_offsets, total_main) = self.controller.with(|c| {
            let item_offsets = measured_entries
                .iter()
                .map(|(item_index, _, _, _)| {
                    c.cache
                        .offset_for(*item_index, self.estimated_item_main, self.item_spacing)
                })
                .collect::<Vec<_>>();
            let total_main = c
                .cache
                .total_main_size(self.estimated_item_main, self.item_spacing);
            (item_offsets, total_main)
        });
        let mut placements = Vec::with_capacity(measured_entries.len());
        for ((item_index, child_id, child_size, _), offset_main) in
            measured_entries.into_iter().zip(item_offsets)
        {
            placements.push((item_index, child_id, offset_main, child_size));
        }

        let total_main_with_padding = total_main + self.padding_main + self.padding_main;
        let cross_with_padding = inner_cross + self.padding_cross + self.padding_cross;
        let size = self
            .axis
            .pack_size(total_main_with_padding, cross_with_padding);
        let needs_scroll_child_size_update =
            self.scroll_controller.with(|c| c.child_size() != size);
        if needs_scroll_child_size_update {
            self.scroll_controller
                .with_mut(|c| c.override_child_size(size));
        }

        let reported_main = clamp_reported_main(
            total_main_with_padding,
            self.viewport_limit,
            self.axis.constraint_max(input.parent_constraint()),
            self.max_viewport_main,
        );

        for placement in &placements {
            let cross_offset = compute_cross_offset(
                inner_cross,
                self.axis.cross(&placement.3),
                self.cross_axis_alignment,
            );
            let mut main_offset = placement.2 + self.padding_main - scroll_offset;
            if self.is_sticky(placement.0) {
                let sticky_start = self.padding_main;
                main_offset = main_offset.max(sticky_start);
                if let Some(next_index) = self.next_sticky_after(placement.0) {
                    let next_offset = self.controller.with(|c| {
                        c.cache
                            .offset_for(next_index, self.estimated_item_main, self.item_spacing)
                    });
                    let max_offset = next_offset + self.padding_main
                        - self.axis.main(&placement.3)
                        - scroll_offset;
                    main_offset = main_offset.min(max_offset);
                }
            }
            let position = self
                .axis
                .position(main_offset, self.padding_cross + cross_offset);
            result.place_child(placement.1, position);
        }

        Ok(result.with_size(self.axis.pack_size(reported_main, cross_with_padding)))
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.item_spacing == other.item_spacing
            && self.estimated_item_main == other.estimated_item_main
            && self.max_viewport_main == other.max_viewport_main
            && self.padding_main == other.padding_main
            && self.padding_cross == other.padding_cross
            && self.viewport_limit == other.viewport_limit
            && self.visible_item_indices == other.visible_item_indices
            && self.composed_item_indices == other.composed_item_indices
            && self.sticky_indices == other.sticky_indices
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other) && self.scroll_offset == other.scroll_offset
    }

    fn place(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != self.composed_item_indices.len() {
            return None;
        }

        let mut placements = Vec::with_capacity(self.visible_item_indices.len());
        let scroll_offset = self.axis.scroll_offset(
            self.scroll_controller
                .with(|controller| controller.child_position()),
        );
        let mut inner_cross = Px::ZERO;
        for (&item_index, &child) in self.composed_item_indices.iter().zip(children.iter()) {
            if self
                .visible_item_indices
                .binary_search(&item_index)
                .is_err()
            {
                continue;
            }
            let child_size = child.size();
            let measured_cross = self.axis.cross(&child_size);
            inner_cross = inner_cross.max(measured_cross);
            let offset_main = self.controller.with(|c| {
                c.cache
                    .offset_for(item_index, self.estimated_item_main, self.item_spacing)
            });
            placements.push((item_index, child, offset_main, child_size));
        }

        placements.sort_unstable_by_key(|placement| placement.0);
        for placement in &placements {
            let cross_offset = compute_cross_offset(
                inner_cross,
                self.axis.cross(&placement.3),
                self.cross_axis_alignment,
            );
            let mut main_offset = placement.2 + self.padding_main - scroll_offset;
            if self.is_sticky(placement.0) {
                let sticky_start = self.padding_main;
                main_offset = main_offset.max(sticky_start);
                if let Some(next_index) = self.next_sticky_after(placement.0) {
                    let next_offset = self.controller.with(|c| {
                        c.cache
                            .offset_for(next_index, self.estimated_item_main, self.item_spacing)
                    });
                    let max_offset = next_offset + self.padding_main
                        - self.axis.main(&placement.3)
                        - scroll_offset;
                    main_offset = main_offset.min(max_offset);
                }
            }
            let position = self
                .axis
                .position(main_offset, self.padding_cross + cross_offset);
            result.place_child(placement.1, position);
        }

        Some(result.into_placement_result())
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
) -> VisibleChildrenPlan {
    if total_count == 0 {
        return VisibleChildrenPlan::empty();
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

    let visible_range = start_index..end_index;
    let mut result = plan.visible_children(visible_range.clone());
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
    VisibleChildrenPlan {
        children: result,
        range: visible_range,
    }
}

fn compute_cross_offset(final_cross: Px, child_cross: Px, alignment: CrossAxisAlignment) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (final_cross - child_cross).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (final_cross - child_cross).max(Px::ZERO),
    }
}

#[tessera]
fn retained_lazy_item(
    index: usize,
    local_index: usize,
    builder: RenderSlotWith<usize>,
    controller: State<LazyListController>,
    retain_focus: bool,
) {
    let scope = remember(FocusScopeNode::new).get();
    let binding = remember(|| {
        (
            index,
            local_index,
            builder.clone(),
            controller,
            retain_focus,
        )
    });
    let binding_changed = binding.with(|binding| {
        binding.0 != index
            || binding.1 != local_index
            || binding.2 != builder
            || binding.3 != controller
            || binding.4 != retain_focus
    });
    if binding_changed {
        if retain_focus
            && scope.has_focus()
            && controller.with(|controller| controller.focused_item != Some(index))
        {
            controller.with_mut(|controller| controller.focused_item = Some(index));
        }
        binding.set((index, local_index, builder, controller, retain_focus));
    }
    let modifier = Modifier::new().focus_scope(Some(scope));
    let modifier = if retain_focus {
        modifier.on_focus_changed(move |focus: tessera_ui::FocusState| {
            if !binding.is_alive() {
                return;
            }
            let (index, _, _, controller, _) = binding.get();
            if !controller.is_alive()
                || !controller.with(|controller| controller.retains_composition)
            {
                return;
            }
            let previous = controller.with(|controller| controller.focused_item);
            let next = if focus.has_focus() {
                Some(index)
            } else if previous == Some(index) {
                None
            } else {
                previous
            };
            if previous != next {
                controller.with_mut(|controller| controller.focused_item = next);
            }
        })
    } else {
        modifier
    };
    layout(Stack).modifier(modifier).child(move || {
        let (_, local_index, builder, _, _) = binding.get();
        builder.render(local_index);
    });
}

#[derive(Clone)]
struct VisibleChildrenPlan {
    children: Vec<VisibleChild>,
    range: Range<usize>,
}

impl VisibleChildrenPlan {
    fn empty() -> Self {
        Self {
            children: Vec::new(),
            range: 0..0,
        }
    }
}

#[derive(Clone, PartialEq)]
struct LazyListFocusArgs {
    axis: LazyAxis,
    controller: State<LazyListController>,
    scroll_controller: State<ScrollableController>,
    total_count: usize,
    total_main: Px,
    viewport_span: Px,
    scroll_offset: Px,
    visible_range: Range<usize>,
    estimated_item_main: Px,
    item_spacing: Px,
}

fn lazy_list_focus_beyond_bounds_modifier(args: State<LazyListFocusArgs>) -> Modifier {
    Modifier::new().focus_beyond_bounds_handler(CallbackWith::new(move |direction| {
        if !args.is_alive() {
            return false;
        }
        let args = args.get();
        let max_scroll = (args.total_main - args.viewport_span).max(Px::ZERO);
        let Some(forward) = args.axis.focus_forward(direction) else {
            return false;
        };
        if args.total_count == 0 || args.viewport_span <= Px::ZERO {
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

        let (target_offset, target_main) = args.controller.with(|c| {
            (
                c.cache
                    .offset_for(target_index, args.estimated_item_main, args.item_spacing),
                c.cache
                    .measurement(target_index)
                    .unwrap_or(args.estimated_item_main),
            )
        });

        let desired_scroll = if forward {
            (target_offset + target_main - args.viewport_span).max(Px::ZERO)
        } else {
            target_offset
        }
        .min(max_scroll);

        if desired_scroll == args.scroll_offset {
            return false;
        }

        let position = args.axis.scroll_position(desired_scroll);
        args.scroll_controller
            .with_mut(|c| c.set_scroll_position(position));
        args.controller
            .with_mut(|c| c.scroll.set_scroll_position(position));
        true
    }))
}

#[derive(Clone, PartialEq)]
enum LazySlot {
    Items(LazyItemsSlot),
    Sticky(LazyStickySlot),
}

impl LazySlot {
    fn items(
        count: usize,
        builder: RenderSlotWith<usize>,
        key_provider: Option<CallbackWith<usize, u64>>,
    ) -> Self {
        Self::Items(LazyItemsSlot {
            count,
            builder,
            key_provider,
        })
    }

    fn sticky(builder: RenderSlotWith<usize>, key_hash: Option<u64>) -> Self {
        Self::Sticky(LazyStickySlot { builder, key_hash })
    }

    fn len(&self) -> usize {
        match self {
            Self::Items(slot) => slot.count,
            Self::Sticky(_) => 1,
        }
    }
}

#[derive(Clone, PartialEq)]
struct LazyStickySlot {
    builder: RenderSlotWith<usize>,
    key_hash: Option<u64>,
}

#[derive(Clone, PartialEq)]
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
                    provider.call(local_index)
                } else {
                    hash_key(index)
                }
            }
            ResolvedSlot::Sticky(slot) => slot.key_hash.unwrap_or_else(|| hash_key(index)),
        };
        let (builder, local_index) = match resolved {
            ResolvedSlot::Items(slot, local_index) => (slot.builder, local_index),
            ResolvedSlot::Sticky(slot) => (slot.builder, 0),
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

#[derive(Clone, PartialEq)]
struct LazySlotEntry {
    start: usize,
    len: usize,
    slot: LazySlot,
}

#[derive(Clone, PartialEq)]
struct VisibleChild {
    item_index: usize,
    local_index: usize,
    builder: RenderSlotWith<usize>,
    key_hash: u64,
}

#[derive(PartialEq, Default)]
struct LazyListCache {
    total_items: usize,
    estimated_main: Px,
    density: f64,
    measured_main: Vec<Option<MeasuredMain>>,
    fenwick: FenwickTree,
}

impl LazyListCache {
    fn configure(&mut self, count: usize, estimated: Px) {
        let density = tessera_ui::dp::scale_factor();
        let density_changed = self.density != density;
        if self.total_items != count {
            self.total_items = count;
            self.measured_main = vec![None; count];
            self.fenwick.resize(count);
        }
        if self.estimated_main != estimated || density_changed {
            self.estimated_main = estimated;
            self.density = density;
            self.fenwick.resize(count);
            for (index, actual) in self.measured_main.iter_mut().enumerate() {
                if let Some(actual) = actual {
                    if density_changed {
                        actual.rebase(density);
                    }
                    self.fenwick.update(index, actual.pixels() - estimated);
                }
            }
        }
    }

    fn measurement(&self, index: usize) -> Option<Px> {
        self.measured_main
            .get(index)
            .copied()
            .flatten()
            .map(MeasuredMain::pixels)
    }

    fn record_measurement(&mut self, index: usize, actual: Px, estimated: Px) {
        if index >= self.total_items {
            return;
        }
        let previous = self.measurement(index);
        if previous == Some(actual) {
            return;
        }

        let prev_delta = previous.map(|val| val - estimated).unwrap_or(Px::ZERO);
        let new_delta = actual - estimated;
        let delta_change = new_delta - prev_delta;
        self.measured_main[index] = Some(MeasuredMain::new(actual, self.density));
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

fn ensure_positive_px(px: Px) -> Px {
    if px <= Px::ZERO { Px(1) } else { px }
}

fn apply_cross_padding(constraint: &mut Constraint, axis: LazyAxis, padding: Px) {
    let total_padding = padding + padding;
    match axis {
        LazyAxis::Vertical => {
            constraint.width = shrink_axis_max(constraint.width, total_padding);
        }
        LazyAxis::Horizontal => {
            constraint.height = shrink_axis_max(constraint.height, total_padding);
        }
    }
}

fn shrink_axis_max(axis: AxisConstraint, amount: Px) -> AxisConstraint {
    let min = axis.min;
    let max = axis.max.map(|m| saturating_sub_px(m, amount));
    if max == Some(min) {
        AxisConstraint::exact(saturating_sub_px(min, amount))
    } else {
        AxisConstraint::new(min, max)
    }
}

fn saturating_sub_px(lhs: Px, rhs: Px) -> Px {
    Px(lhs.0.saturating_sub(rhs.0))
}

#[cfg(test)]
mod tests {

    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        AxisConstraint, ComputedData, Dp, LayoutResult, MeasurementError, Modifier,
        NoopDrawModifier, Px, PxPosition, RenderSlotWith, State, layout::MeasureScope, remember,
        remember_with_key, tessera,
    };

    use crate::{
        alignment::CrossAxisAlignment,
        text::text,
        theme::{MaterialTheme, material_theme},
    };

    use super::{LazyListCache, LazyListController, lazy_column, lazy_row};

    #[tessera]
    fn capped_list_fixture(horizontal: bool) {
        material_theme().child(move || {
            if horizontal {
                lazy_row()
                    .modifier(Modifier::new().size(Dp(360.0), Dp(40.0)))
                    .max_viewport_main(Px(360))
                    .estimated_item_size(Dp(336.0))
                    .items(20, |index| {
                        fixed_test_box()
                            .tag(format!("capped_{index}"))
                            .width(336)
                            .height(40);
                    });
            } else {
                lazy_column()
                    .modifier(Modifier::new().size(Dp(40.0), Dp(360.0)))
                    .max_viewport_main(Px(360))
                    .estimated_item_size(Dp(336.0))
                    .items(20, |index| {
                        fixed_test_box()
                            .tag(format!("capped_{index}"))
                            .width(40)
                            .height(336);
                    });
            }
        });
    }

    #[test]
    fn viewport_cap_limits_initial_item_construction_on_both_axes() {
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::viewport_cap_limits_initial_item_construction_on_both_axes",
            || {
                for horizontal in [false, true] {
                    let mut session = tessera_ui::testing::start_layout_test_session(
                        tessera_ui::testing::layout_test(move || {
                            capped_list_fixture(horizontal);
                        })
                        .viewport_px(360, 360),
                    );
                    let snapshot = tessera_ui::testing::pump_layout_test_session(&mut session, 0);
                    snapshot.node("capped_0").exists();
                    snapshot.node("capped_1").exists();
                    assert_eq!(
                        session
                            .work()
                            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build),
                        2,
                        "the first layout must only construct the two intersecting items"
                    );
                }
            },
        );
    }

    #[derive(Clone, Copy, Default)]
    struct FixtureHandles {
        estimate: Option<State<Dp>>,
        collapsed_width: Option<State<Dp>>,
        collapsed_list: Option<State<LazyListController>>,
        programmatic_list: Option<State<LazyListController>>,
        responsive_cross: Option<State<Dp>>,
        responsive_main: Option<State<Dp>>,
        list_generation: Option<State<u64>>,
        replaced_list: Option<State<LazyListController>>,
    }

    #[tessera]
    fn programmatic_list_fixture(
        horizontal: bool,
        observed: tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let controller = remember(LazyListController::new);
        observed.with_mut(|handles| handles.programmatic_list = Some(controller));
        let items = RenderSlotWith::new(|index| {
            fixed_test_box()
                .tag(format!("programmatic_{index}"))
                .width(20)
                .height(20);
        });
        material_theme().child(move || {
            if horizontal {
                lazy_row()
                    .modifier(Modifier::new().size(Dp(100.0), Dp(40.0)))
                    .controller(controller)
                    .estimated_item_size(Dp(20.0))
                    .items_with_key_shared(100, |index| index, items);
            } else {
                lazy_column()
                    .modifier(Modifier::new().size(Dp(40.0), Dp(100.0)))
                    .controller(controller)
                    .estimated_item_size(Dp(20.0))
                    .items_with_key_shared(100, |index| index, items);
            }
        });
    }

    fn check_programmatic_position(horizontal: bool) {
        tessera_ui::dp::set_scale_factor(1.0);
        let observed = tessera_ui::testing::TestProbe::default();
        let fixture_observed = observed.clone();
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(move || {
                programmatic_list_fixture(horizontal, fixture_observed.clone());
            })
            .viewport_px(120, 120),
        );
        tessera_ui::testing::pump_layout_test_session(&mut session, 0);
        let controller = observed.get().programmatic_list.expect("controller");
        controller.with_mut(|controller| {
            controller.scroll_to(
                if horizontal { Dp(80.0) } else { Dp::ZERO },
                if horizontal { Dp::ZERO } else { Dp(80.0) },
            )
        });
        tessera_ui::testing::pump_layout_test_session(&mut session, 16_666_667);
        let snapshot = tessera_ui::testing::pump_layout_test_session(&mut session, 33_333_334);
        snapshot.node("programmatic_4").position(0, 0);
        controller.with_mut(|controller| controller.scroll_to(Dp::ZERO, Dp::ZERO));
        tessera_ui::testing::pump_layout_test_session(&mut session, 50_000_001);
        let snapshot = tessera_ui::testing::pump_layout_test_session(&mut session, 66_666_668);
        snapshot.node("programmatic_0").position(0, 0);
    }

    #[test]
    fn lazy_column_applies_programmatic_positions_after_mounting() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        check_programmatic_position(false);
    }

    #[test]
    fn lazy_row_applies_programmatic_positions_after_mounting() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        check_programmatic_position(true);
    }

    #[tessera]
    fn replaceable_list_fixture(
        horizontal: bool,
        observed: tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let generation = remember(|| 0_u64);
        observed.with_mut(|handles| handles.list_generation = Some(generation));
        let controller = remember_with_key(generation.get(), LazyListController::new);
        observed.with_mut(|handles| handles.replaced_list = Some(controller));
        let items = RenderSlotWith::new(|index| {
            fixed_test_box()
                .tag(format!("replacement_{index}"))
                .width(20)
                .height(20);
        });
        material_theme().child(move || {
            if horizontal {
                lazy_row()
                    .modifier(Modifier::new().size(Dp(100.0), Dp(40.0)))
                    .controller(controller)
                    .estimated_item_size(Dp(20.0))
                    .items_with_key_shared(100, |index| index, items);
            } else {
                lazy_column()
                    .modifier(Modifier::new().size(Dp(40.0), Dp(100.0)))
                    .controller(controller)
                    .estimated_item_size(Dp(20.0))
                    .items_with_key_shared(100, |index| index, items);
            }
        });
    }

    #[test]
    fn replacing_lazy_list_controllers_refreshes_retained_layout_and_scroll_ownership() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::replacing_lazy_list_controllers_refreshes_retained_layout_and_scroll_ownership",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                for horizontal in [false, true] {
                    let observed = tessera_ui::testing::TestProbe::default();
                    let fixture_observed = observed.clone();
                    let mut session = tessera_ui::testing::start_layout_test_session(
                        tessera_ui::testing::layout_test(move || {
                            replaceable_list_fixture(horizontal, fixture_observed.clone());
                        })
                        .viewport_px(120, 120),
                    );
                    tessera_ui::testing::pump_layout_test_session(&mut session, 0);
                    let mut time = 0;
                    for generation in 1..=4 {
                        let previous = observed.get().replaced_list.unwrap();
                        observed.get().list_generation.unwrap().set(generation);
                        time += 16_666_667;
                        tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        time += 16_666_667;
                        let snapshot =
                            tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        assert!(
                            !previous.is_alive(),
                            "the old keyed controller was actually released"
                        );
                        let current = observed.get().replaced_list.unwrap();
                        assert!(current.is_alive());
                        assert!(
                            current != previous,
                            "the controller identity must change with its key"
                        );
                        snapshot.node("replacement_0").position(0, 0);
                        current.with_mut(|controller| {
                            controller.scroll_to(
                                if horizontal { Dp(80.0) } else { Dp::ZERO },
                                if horizontal { Dp::ZERO } else { Dp(80.0) },
                            )
                        });
                        time += 16_666_667;
                        tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        time += 16_666_667;
                        let snapshot =
                            tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        snapshot.node("replacement_4").position(0, 0);
                        current.with_mut(|controller| controller.scroll_to(Dp::ZERO, Dp::ZERO));
                        time += 16_666_667;
                        tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        time += 16_666_667;
                        let snapshot =
                            tessera_ui::testing::pump_layout_test_session(&mut session, time);
                        snapshot.node("replacement_0").position(0, 0);
                    }
                }
            },
        );
    }

    #[derive(Clone, PartialEq)]
    struct ResponsiveItemLayout(bool, bool);

    impl tessera_ui::LayoutPolicy for ResponsiveItemLayout {
        fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let cross = if self.0 {
                input.parent_constraint().height().resolve_max()
            } else {
                input.parent_constraint().width().resolve_max()
            }
            .expect("bounded cross axis");
            let main = if cross < Dp(200.0).to_px() {
                Dp(80.0).to_px()
            } else {
                Dp(40.0).to_px()
            };
            let main = if self.1 { main * 10 } else { main };
            Ok(LayoutResult::new(if self.0 {
                ComputedData {
                    width: main,
                    height: cross,
                }
            } else {
                ComputedData {
                    width: cross,
                    height: main,
                }
            }))
        }
    }

    #[tessera]
    fn responsive_anchor_fixture(
        horizontal: bool,
        tall: bool,
        observed: tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let cross = remember(|| Dp(300.0));
        let main = remember(|| Dp(120.0));
        let controller = remember(LazyListController::new);
        observed.with_mut(|handles| handles.responsive_cross = Some(cross));
        observed.with_mut(|handles| handles.responsive_main = Some(main));
        observed.with_mut(|handles| handles.programmatic_list = Some(controller));
        let items = RenderSlotWith::new(move |index| {
            tessera_ui::layout::layout(ResponsiveItemLayout(horizontal, tall)).modifier(
                Modifier::new().semantics(SemanticsArgs {
                    test_tag: Some(format!("responsive_{index}")),
                    ..Default::default()
                }),
            );
        });
        material_theme().child(move || {
            crate::boxed::boxed().children(move || {
                if horizontal {
                    lazy_row()
                        .modifier(Modifier::new().size(main.get(), cross.get()))
                        .scrollbar_behavior(crate::scrollable::ScrollBarBehavior::Hidden)
                        .controller(controller)
                        .overscan(2)
                        .estimated_item_size(Dp(40.0))
                        .items_with_key_shared(20, |index| index, items);
                } else {
                    lazy_column()
                        .modifier(Modifier::new().size(cross.get(), main.get()))
                        .scrollbar_behavior(crate::scrollable::ScrollBarBehavior::Hidden)
                        .controller(controller)
                        .overscan(2)
                        .estimated_item_size(Dp(40.0))
                        .items_with_key_shared(20, |index| index, items);
                }
            });
        });
    }

    #[test]
    fn responsive_measurements_preserve_the_first_visible_item_start_on_both_axes() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::responsive_measurements_preserve_the_first_visible_item_start_on_both_axes",
            || {
                for density in [1.0, 1.25, 1.5, 2.0] {
                    tessera_ui::dp::set_scale_factor(density);
                    for (horizontal, tall) in
                        [(false, false), (true, false), (false, true), (true, true)]
                    {
                        let observed = tessera_ui::testing::TestProbe::default();
                        let fixture_observed = observed.clone();
                        let mut session = tessera_ui::testing::start_layout_test_session(
                            tessera_ui::testing::layout_test(move || {
                                responsive_anchor_fixture(
                                    horizontal,
                                    tall,
                                    fixture_observed.clone(),
                                );
                            })
                            .viewport_px(800, 800),
                        );
                        tessera_ui::testing::pump_layout_test_session(&mut session, 0);
                        let controller = observed.get().programmatic_list.expect("list controller");
                        let offset = if tall { Dp(60.0) } else { Dp(180.0) };
                        controller.with_mut(|controller| {
                            controller.scroll_to(
                                if horizontal { offset } else { Dp::ZERO },
                                if horizontal { Dp::ZERO } else { offset },
                            );
                        });
                        let cross = observed.get().responsive_cross.expect("cross-axis state");
                        let main_span = observed.get().responsive_main.expect("main-axis state");
                        let mut frame = 1;
                        for (width, span) in [
                            (300.0, 120.0),
                            (150.0, 60.0),
                            (300.0, 120.0),
                            (150.0, 60.0),
                            (300.0, 120.0),
                        ] {
                            cross.set(Dp(width));
                            main_span.set(Dp(span));
                            let main = Dp(if width < 200.0 { 80.0 } else { 40.0 }).to_px().0
                                * if tall { 10 } else { 1 };
                            let position = Dp(if tall { -60.0 } else { 20.0 }).to_px().0;
                            let cross_size = Dp(width).to_px().0;
                            for _ in 0..4 {
                                let snapshot = tessera_ui::testing::pump_layout_test_session(
                                    &mut session,
                                    frame * 16_666_667,
                                );
                                snapshot
                                    .node(if tall { "responsive_0" } else { "responsive_5" })
                                    .position(
                                        if horizontal { position } else { 0 },
                                        if horizontal { 0 } else { position },
                                    )
                                    .size(
                                        if horizontal { main } else { cross_size },
                                        if horizontal { cross_size } else { main },
                                    );
                                frame += 1;
                            }
                        }
                    }
                }
            },
        );
    }

    #[tessera]
    fn collapsed_viewport_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let width = remember(|| Dp(300.0));
        let controller = remember(LazyListController::new);
        observed.with_mut(|handles| handles.collapsed_width = Some(width));
        observed.with_mut(|handles| handles.collapsed_list = Some(controller));
        material_theme().child(move || {
            lazy_column()
                .modifier(Modifier::new().width(width.get()).height(Dp(240.0)))
                .controller(controller)
                .estimated_item_size(Dp(60.0))
                .items(3, |_| {
                    text("A responsive list retains measured geometry while its viewport is temporarily collapsed.");
                });
        });
    }

    #[tessera]
    fn density_scroll_fixture(
        horizontal: bool,
        observed: tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let controller = remember(LazyListController::new);
        observed.with_mut(|handles| handles.programmatic_list = Some(controller));
        let items = RenderSlotWith::new(|index| {
            fixed_test_box()
                .tag(format!("density_{index}"))
                .width(Dp(20.0).to_px().0)
                .height(Dp(40.0).to_px().0);
        });
        material_theme().child(move || {
            crate::boxed::boxed().children(move || {
                if horizontal {
                    lazy_row()
                        .modifier(Modifier::new().size(Dp(100.0), Dp(100.0)))
                        .controller(controller)
                        .content_padding(Dp(4.0))
                        .item_spacing(Dp(4.0))
                        .scrollbar_behavior(crate::scrollable::ScrollBarBehavior::Hidden)
                        .estimated_item_size(Dp(20.0))
                        .items_with_key_shared(100, |index| index, items);
                } else {
                    lazy_column()
                        .modifier(Modifier::new().size(Dp(100.0), Dp(100.0)))
                        .controller(controller)
                        .content_padding(Dp(4.0))
                        .item_spacing(Dp(4.0))
                        .scrollbar_behavior(crate::scrollable::ScrollBarBehavior::Hidden)
                        .estimated_item_size(Dp(40.0))
                        .items_with_key_shared(100, |index| index, items);
                }
            });
        });
    }

    #[test]
    fn density_changes_preserve_scrolled_items_and_new_programmatic_requests() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::density_changes_preserve_scrolled_items_and_new_programmatic_requests",
            || {
                for horizontal in [false, true] {
                    tessera_ui::dp::set_scale_factor(1.0);
                    let observed = tessera_ui::testing::TestProbe::default();
                    let fixture_observed = observed.clone();
                    let mut session = tessera_ui::testing::start_layout_test_session(
                        tessera_ui::testing::layout_test(move || {
                            density_scroll_fixture(horizontal, fixture_observed.clone());
                        })
                        .viewport_px(400, 400),
                    );
                    for frame in 0..4 {
                        tessera_ui::testing::pump_layout_test_session(
                            &mut session,
                            frame * 16_666_667,
                        );
                    }
                    let controller = observed.get().programmatic_list.expect("controller");
                    let offset = if horizontal { Dp(104.0) } else { Dp(204.0) };
                    controller.with_mut(|controller| {
                        controller.scroll_to(
                            if horizontal { offset } else { Dp::ZERO },
                            if horizontal { Dp::ZERO } else { offset },
                        )
                    });
                    let mut frame = 4;
                    for density in [1.0, 1.25, 1.5, 2.0, 1.0] {
                        tessera_ui::dp::set_scale_factor(density);
                        if density == 1.5 {
                            controller.with_mut(|controller| {
                                controller.scroll_to(
                                    if horizontal { offset } else { Dp::ZERO },
                                    if horizontal { Dp::ZERO } else { offset },
                                )
                            });
                        }
                        for _ in 0..4 {
                            let snapshot = tessera_ui::testing::pump_layout_test_session(
                                &mut session,
                                frame * 16_666_667,
                            );
                            snapshot
                                .node("density_5")
                                .position(
                                    Dp(if horizontal { 20.0 } else { 4.0 }).to_px().0,
                                    Dp(if horizontal { 4.0 } else { 20.0 }).to_px().0,
                                )
                                .size(Dp(20.0).to_px().0, Dp(40.0).to_px().0);
                            frame += 1;
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn collapsed_cross_axis_preserves_usable_measurements() {
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
        let controller = observed.get().collapsed_list.expect("list controller");
        let before = controller.with(|state| state.cache.measured_main.clone());
        assert!(before.iter().all(Option::is_some));
        let width = observed.get().collapsed_width.expect("viewport width");
        width.set(Dp(0.0));
        for frame in 8..12 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(
            controller.with(|state| state.cache.measured_main.clone()),
            before
        );
        width.set(Dp(300.0));
        for frame in 12..16 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(
            controller.with(|state| state.cache.measured_main.clone()),
            before
        );
    }

    #[test]
    fn estimate_changes_rebase_measured_and_unmeasured_offsets() {
        let mut cache = LazyListCache::default();
        cache.configure(5, Px(100));
        for (index, size) in [(0, 40), (1, 60), (3, 80)] {
            cache.record_measurement(index, Px(size), Px(100));
        }
        assert_eq!(cache.total_main_size(Px(100), Px(8)), Px(412));
        cache.configure(5, Px(20));
        assert_eq!(cache.total_main_size(Px(20), Px(8)), Px(252));
        assert_eq!(cache.offset_for(2, Px(20), Px(8)), Px(116));
        assert_eq!(cache.offset_for(4, Px(20), Px(8)), Px(232));
        assert_eq!(cache.index_for_offset(Px(115), Px(20), Px(8)), 1);
        assert_eq!(cache.index_for_offset(Px(116), Px(20), Px(8)), 2);
        cache.record_measurement(1, Px(75), Px(20));
        cache.configure(5, Px(100));
        assert_eq!(cache.total_main_size(Px(100), Px(8)), Px(427));
        cache.configure(0, Px(20));
        assert_eq!(cache.total_main_size(Px(20), Px(8)), Px::ZERO);
        cache.configure(3, Px(100));
        assert_eq!(cache.total_main_size(Px(100), Px(8)), Px(316));
    }

    #[test]
    fn unvisited_measurements_rebase_density_without_round_trip_drift() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::unvisited_measurements_rebase_density_without_round_trip_drift",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let mut cache = LazyListCache::default();
                cache.configure(3, Px(20));
                cache.record_measurement(0, Px(33), Px(20));
                cache.record_measurement(2, Px(55), Px(20));
                for (density, estimated, spacing, offset, total) in [
                    (1.25, 25, 5, 46, 144),
                    (1.5, 30, 6, 55, 173),
                    (2.0, 40, 8, 74, 232),
                    (1.0, 20, 4, 37, 116),
                ] {
                    tessera_ui::dp::set_scale_factor(density);
                    cache.configure(3, Px(estimated));
                    assert_eq!(cache.offset_for(1, Px(estimated), Px(spacing)), Px(offset));
                    assert_eq!(cache.total_main_size(Px(estimated), Px(spacing)), Px(total));
                }
            },
        );
    }

    #[tessera]
    fn changing_estimate_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let estimate = remember(|| Dp(100.0));
        observed.with_mut(|handles| handles.estimate = Some(estimate));
        material_theme().child(move || {
            lazy_column()
                .modifier(Modifier::new().fill_max_size())
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .estimated_item_size(estimate.get())
                .item_spacing(Dp(8.0))
                .overscan(10)
                .items(4, |index| {
                    fixed_test_box()
                        .tag(format!("estimate_{index}"))
                        .width(20)
                        .height(40);
                });
        });
    }

    #[test]
    fn changing_estimates_and_density_keep_items_separated() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_list::tests::changing_estimates_and_density_keep_items_separated",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let observed = tessera_ui::testing::TestProbe::default();
                let fixture_observed = observed.clone();
                let mut session = tessera_ui::testing::start_layout_test_session(
                    tessera_ui::testing::layout_test(move || {
                        changing_estimate_fixture(fixture_observed.clone());
                    })
                    .viewport_px(200, 240),
                );
                let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 0);
                let estimate = observed.get().estimate.expect("estimate state");
                for (frame, (density, value)) in [(1.0, 20.0), (1.5, 140.0), (1.0, 60.0)]
                    .into_iter()
                    .enumerate()
                {
                    tessera_ui::dp::set_scale_factor(density);
                    estimate.set(Dp(value));
                    let snapshot = tessera_ui::testing::pump_layout_test_session(
                        &mut session,
                        (frame as u64 + 1) * 16_666_667,
                    );
                    for index in 0..4 {
                        snapshot
                            .node(&format!("estimate_{index}"))
                            .position(0, index * (40 + (8.0 * density) as i32));
                    }
                }
                tessera_ui::dp::set_scale_factor(1.0);
            },
        );
    }

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    #[derive(Clone, PartialEq)]
    struct PreferredConstraintTestLayout {
        preferred_width: i32,
        preferred_height: i32,
    }

    impl tessera_ui::LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    impl tessera_ui::LayoutPolicy for PreferredConstraintTestLayout {
        fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let preferred_width = Px::new(self.preferred_width);
            let preferred_height = Px::new(self.preferred_height);
            let width = AxisConstraint::exact(preferred_width)
                .intersect(input.parent_constraint().width())
                .clamp(preferred_width);
            let height = AxisConstraint::exact(preferred_height)
                .intersect(input.parent_constraint().height())
                .clamp(preferred_height);
            Ok(LayoutResult::new(ComputedData { width, height }))
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
    fn preferred_constraint_test_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or(0);
        let height = height.unwrap_or(0);

        tessera_ui::layout::layout(PreferredConstraintTestLayout {
            preferred_width: width,
            preferred_height: height,
        })
        .draw(NoopDrawModifier)
        .modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: Some(tag),
            ..Default::default()
        }));
    }

    #[tessera]
    fn lazy_column_layout_case() {
        lazy_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(50))),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .estimated_item_size(Px::new(10).into())
            .content_padding(Px::new(4).into())
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_first".to_string())
                    .width(20)
                    .height(10);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_second".to_string())
                    .width(18)
                    .height(12);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_third".to_string())
                    .width(16)
                    .height(8);
            });
    }

    #[tessera]
    fn lazy_row_layout_case() {
        lazy_row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(70))),
                Some(AxisConstraint::exact(Px::new(30))),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .estimated_item_size(Px::new(10).into())
            .content_padding(Px::new(4).into())
            .item(|| {
                fixed_test_box()
                    .tag("lazy_row_first".to_string())
                    .width(20)
                    .height(10);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_row_second".to_string())
                    .width(15)
                    .height(12);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_row_third".to_string())
                    .width(18)
                    .height(8);
            });
    }

    #[tessera]
    fn lazy_column_scrolled_layout_case() {
        let controller = remember(LazyListController::new);
        let target_position = PxPosition::new(Px::ZERO, Px::new(-5));
        if controller.with(|c| c.scroll_controller().child_position()) != target_position {
            controller.with_mut(|c| {
                c.scroll_controller_mut()
                    .set_scroll_position(target_position);
            });
        }

        lazy_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(39))),
            ))
            .controller(controller)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .estimated_item_size(Px::new(10).into())
            .content_padding(Px::new(4).into())
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_scrolled_first".to_string())
                    .width(20)
                    .height(10);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_scrolled_second".to_string())
                    .width(18)
                    .height(12);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_scrolled_third".to_string())
                    .width(16)
                    .height(8);
            });
    }

    #[tessera]
    fn lazy_column_sticky_layout_case() {
        let controller = remember(LazyListController::new);
        let target_position = PxPosition::new(Px::ZERO, Px::new(-6));
        if controller.with(|c| c.scroll_controller().child_position()) != target_position {
            controller.with_mut(|c| {
                c.scroll_controller_mut()
                    .set_scroll_position(target_position);
            });
        }

        lazy_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(38))),
            ))
            .controller(controller)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .estimated_item_size(Px::new(10).into())
            .content_padding(Px::new(4).into())
            .sticky_header(|| {
                fixed_test_box()
                    .tag("lazy_column_sticky_header".to_string())
                    .width(20)
                    .height(10);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_sticky_body".to_string())
                    .width(18)
                    .height(12);
            })
            .item(|| {
                fixed_test_box()
                    .tag("lazy_column_sticky_tail".to_string())
                    .width(16)
                    .height(8);
            });
    }

    #[tessera]
    fn lazy_column_non_stretch_cross_axis_case() {
        lazy_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(50))),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .estimated_item_size(Px::new(10).into())
            .content_padding(Px::new(4).into())
            .item(|| {
                preferred_constraint_test_box()
                    .tag("lazy_column_non_stretch".to_string())
                    .width(20)
                    .height(10);
            });
    }

    #[tessera]
    fn lazy_column_stable_visible_items_case() {
        let controller = remember(LazyListController::new);
        // Rebuilds this component on every animated frame.
        let frame_clock = tessera_ui::remember(tessera_ui::current_frame_nanos);
        let _ = frame_clock.get();
        tessera_ui::launch_once(async move {
            let step = move |frame_nanos: u64| {
                let offset = match frame_nanos {
                    0 => 0,
                    16_666_667 => 2,
                    33_333_334 => 4,
                    _ => 4,
                };
                let position = PxPosition::new(Px::ZERO, Px::new(-offset));
                controller.with_mut(|c| {
                    c.scroll_controller_mut().set_scroll_position(position);
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
                let item_width =
                    controller.with(|c| 20 - c.scroll_controller().child_position().y as i32);
                lazy_column()
                    .modifier(Modifier::new().constrain(
                        Some(AxisConstraint::exact(Px::new(60))),
                        Some(AxisConstraint::exact(Px::new(50))),
                    ))
                    .controller(controller)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .estimated_item_size(Px::new(20).into())
                    .items(10, move |index| {
                        fixed_test_box()
                            .tag(format!("stable_visible_item_{index}"))
                            .width(item_width)
                            .height(20);
                    });
            });
    }

    #[test]
    fn lazy_column_positions_items_with_padding_and_spacing() {
        tessera_ui::assert_layout! {
            viewport: (80, 60),
            content: {
                material_theme().child(|| { lazy_column_layout_case(); });
            },
            expect: {
                node("lazy_column_first").position(4, 4).size(20, 10);
                node("lazy_column_second").position(4, 17).size(18, 12);
                node("lazy_column_third").position(4, 32).size(16, 8);
            }
        }
    }

    #[test]
    fn lazy_row_positions_items_with_padding_and_spacing() {
        tessera_ui::assert_layout! {
            viewport: (90, 40),
            content: {
                material_theme().child(|| { lazy_row_layout_case(); });
            },
            expect: {
                node("lazy_row_first").position(4, 4).size(20, 10);
                node("lazy_row_second").position(27, 4).size(15, 12);
                node("lazy_row_third").position(45, 4).size(18, 8);
            }
        }
    }

    #[test]
    fn lazy_column_scroll_offset_repositions_visible_items() {
        tessera_ui::assert_layout! {
            viewport: (80, 60),
            content: {
                material_theme().child(|| { lazy_column_scrolled_layout_case(); });
            },
            expect: {
                node("lazy_column_scrolled_first").position(4, -1).size(20, 10);
                node("lazy_column_scrolled_second").position(4, 12).size(18, 12);
                node("lazy_column_scrolled_third").position(4, 27).size(16, 8);
            }
        }
    }

    #[test]
    fn lazy_column_sticky_header_stays_pinned_to_padding() {
        tessera_ui::assert_layout! {
            viewport: (80, 50),
            content: {
                material_theme().child(|| { lazy_column_sticky_layout_case(); });
            },
            expect: {
                node("lazy_column_sticky_header").position(4, 4).size(20, 10);
                node("lazy_column_sticky_body").position(4, 11).size(18, 12);
                node("lazy_column_sticky_tail").position(4, 26).size(16, 8);
            }
        }
    }

    #[test]
    fn lazy_column_non_stretch_cross_axis_does_not_force_exact_parent_width() {
        tessera_ui::assert_layout! {
            viewport: (80, 60),
            content: {
                material_theme().child(|| { lazy_column_non_stretch_cross_axis_case(); });
            },
            expect: {
                node("lazy_column_non_stretch").position(4, 4).size(20, 10);
            }
        }
    }

    #[test]
    fn lazy_column_rebuilds_visible_items_when_item_slot_updates() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                lazy_column_stable_visible_items_case();
            })
            .viewport_px(80, 60),
        );

        tessera_ui::testing::pump_layout_test_session(&mut session, 0)
            .node("stable_visible_item_0")
            .width(20);
        let first_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert!(first_builds > 0);

        tessera_ui::testing::pump_layout_test_session(&mut session, 16_666_667)
            .node("stable_visible_item_0")
            .width(22);
        let second_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert!(second_builds > 0);

        tessera_ui::testing::pump_layout_test_session(&mut session, 33_333_334)
            .node("stable_visible_item_0")
            .width(24);
        let third_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert!(third_builds > 0);
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "lazy_list_retention_tests.rs"]
mod retention_tests;
