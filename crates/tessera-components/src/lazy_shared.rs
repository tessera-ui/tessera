//! Shared measurements and viewport bounds for lazy scrolling components.
//!
//! ## Usage
//!
//! Display long lists and grids in resizable windows on different displays.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use tessera_ui::{
    AxisConstraint, CallbackWith, ComputedData, Constraint, FocusDirection, ParentConstraint, Px,
    PxPosition, RenderSlotWith,
};

use crate::{
    alignment::CrossAxisAlignment,
    layout_flex::{cross_axis_child_constraint, px_from_i64},
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LazyAxis {
    #[default]
    Vertical,
    Horizontal,
}

impl LazyAxis {
    pub(crate) fn main(self, size: &ComputedData) -> Px {
        match self {
            Self::Vertical => size.height,
            Self::Horizontal => size.width,
        }
    }

    pub(crate) fn cross(self, size: &ComputedData) -> Px {
        match self {
            Self::Vertical => size.width,
            Self::Horizontal => size.height,
        }
    }

    pub(crate) fn pack_size(self, main: Px, cross: Px) -> ComputedData {
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

    pub(crate) fn position(self, main: Px, cross: Px) -> PxPosition {
        match self {
            Self::Vertical => PxPosition::new(cross, main),
            Self::Horizontal => PxPosition::new(main, cross),
        }
    }

    pub(crate) fn scroll_position(self, offset: Px) -> PxPosition {
        self.position(-offset, Px::ZERO)
    }

    pub(crate) fn scroll_offset(self, position: PxPosition) -> Px {
        let main = match self {
            Self::Vertical => position.y,
            Self::Horizontal => position.x,
        };
        Px((-main).max(0.0) as i32)
    }

    pub(crate) fn focus_forward(self, direction: FocusDirection) -> Option<bool> {
        match (self, direction) {
            (_, FocusDirection::Next | FocusDirection::Enter) => Some(true),
            (_, FocusDirection::Previous | FocusDirection::Exit) => Some(false),
            (Self::Vertical, FocusDirection::Down) => Some(true),
            (Self::Vertical, FocusDirection::Up) => Some(false),
            (Self::Horizontal, FocusDirection::Right) => Some(true),
            (Self::Horizontal, FocusDirection::Left) => Some(false),
            _ => None,
        }
    }

    pub(crate) fn child_constraint(
        self,
        cross: AxisConstraint,
        alignment: CrossAxisAlignment,
    ) -> Constraint {
        let cross = cross_axis_child_constraint(cross, alignment);
        match self {
            Self::Vertical => Constraint::new(cross, AxisConstraint::NONE),
            Self::Horizontal => Constraint::new(AxisConstraint::NONE, cross),
        }
    }

    pub(crate) fn cross_constraint(self, parent: ParentConstraint<'_>) -> AxisConstraint {
        match self {
            Self::Vertical => parent.width(),
            Self::Horizontal => parent.height(),
        }
    }

    pub(crate) fn constraint_max(self, parent: ParentConstraint<'_>) -> Option<Px> {
        match self {
            Self::Vertical => parent.height().resolve_max(),
            Self::Horizontal => parent.width().resolve_max(),
        }
    }
}

pub(crate) fn hash_key(key: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, PartialEq)]
pub(crate) struct LazyItemsSlot {
    pub(crate) count: usize,
    pub(crate) builder: RenderSlotWith<usize>,
    pub(crate) key_provider: Option<CallbackWith<usize, u64>>,
}

impl LazyItemsSlot {
    pub(crate) fn items(
        count: usize,
        builder: RenderSlotWith<usize>,
        key_provider: Option<CallbackWith<usize, u64>>,
    ) -> Self {
        Self {
            count,
            builder,
            key_provider,
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct LazyItemsPlan {
    entries: Vec<(usize, LazyItemsSlot)>,
    total_count: usize,
}

impl LazyItemsPlan {
    pub(crate) fn new(slots: Vec<LazyItemsSlot>) -> Self {
        let mut total_count = 0;
        let entries = slots
            .into_iter()
            .map(|slot| {
                let start = total_count;
                total_count += slot.count;
                (start, slot)
            })
            .collect();
        Self {
            entries,
            total_count,
        }
    }

    pub(crate) fn total_count(&self) -> usize {
        self.total_count
    }

    pub(crate) fn resolve(&self, index: usize) -> Option<(&LazyItemsSlot, usize)> {
        self.entries.iter().find_map(|(start, slot)| {
            (*start..*start + slot.count)
                .contains(&index)
                .then(|| (slot, index - *start))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredMain {
    pixels: Px,
    logical: f64,
}

impl MeasuredMain {
    pub(crate) fn new(pixels: Px, density: f64) -> Self {
        Self {
            pixels,
            logical: f64::from(pixels.0) / density,
        }
    }

    pub(crate) fn pixels(self) -> Px {
        self.pixels
    }

    pub(crate) fn rebase(&mut self, density: f64) {
        self.pixels = Px((self.logical * density) as i32);
    }
}

/// Clamps the main-axis size a lazy component reports to its parent.
///
/// The reported size never exceeds the viewport, and is further limited by the
/// parent constraint for the main axis, falling back to `fallback_limit` when
/// the parent does not constrain that axis.
pub(crate) fn clamp_reported_main(
    total_main: Px,
    viewport_span: Px,
    constraint_max: Option<Px>,
    fallback_limit: Option<Px>,
) -> Px {
    let viewport = viewport_span.max(Px::ZERO);
    let mut report = total_main.min(viewport);
    if let Some(max_value) = constraint_max.or(fallback_limit) {
        report = report.min(max_value.max(Px::ZERO));
    }
    report
}

macro_rules! impl_lazy_builder {
    ($builder:ident, $content:ident) => {
        impl $builder {
            /// Append a single lazy item.
            pub fn item<F, Output>(&self, builder: F) -> Self
            where
                F: Fn() -> Output + Send + Sync + 'static,
            {
                self.items(1, move |_| builder())
            }

            /// Append a single lazy item with a stable key.
            pub fn item_with_key<K, F, Output>(&self, key: K, builder: F) -> Self
            where
                K: Hash,
                F: Fn() -> Output + Send + Sync + 'static,
            {
                let key = hash_key(key);
                self.props.append("content", move |props| {
                    props
                        .content
                        .get_or_insert_with($content::default)
                        .push(LazySlot::items(
                            1,
                            RenderSlotWith::new(move |_| builder()),
                            Some(CallbackWith::new(move |_| key)),
                        ));
                });
                Self { props: self.props }
            }

            /// Append a batch of indexed lazy items.
            pub fn items<F, Output>(&self, count: usize, builder: F) -> Self
            where
                F: Fn(usize) -> Output + Send + Sync + 'static,
            {
                self.props.append("content", move |props| {
                    props
                        .content
                        .get_or_insert_with($content::default)
                        .push(LazySlot::items(count, RenderSlotWith::new(builder), None));
                });
                Self { props: self.props }
            }

            /// Append indexed lazy items with stable keys.
            pub fn items_with_key<K, KF, F, Output>(
                &self,
                count: usize,
                key_provider: KF,
                builder: F,
            ) -> Self
            where
                K: Hash,
                KF: Fn(usize) -> K + Send + Sync + 'static,
                F: Fn(usize) -> Output + Send + Sync + 'static,
            {
                self.props.append("content", move |props| {
                    props
                        .content
                        .get_or_insert_with($content::default)
                        .push(LazySlot::items(
                            count,
                            RenderSlotWith::new(builder),
                            Some(CallbackWith::new(move |index| {
                                hash_key(key_provider(index))
                            })),
                        ));
                });
                Self { props: self.props }
            }

            /// Append lazy items from an iterator, exposing index and element.
            pub fn items_from_iter<I, T, F, Output>(&self, iter: I, builder: F) -> Self
            where
                I: IntoIterator<Item = T>,
                T: Send + Sync + 'static,
                F: Fn(usize, &T) -> Output + Send + Sync + 'static,
            {
                let items: Vec<T> = iter.into_iter().collect();
                self.props.append("content", move |props| {
                    if !items.is_empty() {
                        props
                            .content
                            .get_or_insert_with($content::default)
                            .push(LazySlot::items(
                                items.len(),
                                RenderSlotWith::new(move |index| builder(index, &items[index])),
                                None,
                            ));
                    }
                });
                Self { props: self.props }
            }

            /// Append lazy items from an iterator with stable keys.
            pub fn items_from_iter_with_key<I, T, K, KF, F, Output>(
                &self,
                iter: I,
                key_provider: KF,
                builder: F,
            ) -> Self
            where
                I: IntoIterator<Item = T>,
                T: Send + Sync + 'static,
                K: Hash,
                KF: Fn(usize, &T) -> K + Send + Sync + 'static,
                F: Fn(usize, &T) -> Output + Send + Sync + 'static,
            {
                let items: Arc<Vec<T>> = Arc::new(iter.into_iter().collect());
                self.props.append("content", move |props| {
                    if !items.is_empty() {
                        let keys = items.clone();
                        props
                            .content
                            .get_or_insert_with($content::default)
                            .push(LazySlot::items(
                                items.len(),
                                RenderSlotWith::new(move |index| builder(index, &items[index])),
                                Some(CallbackWith::new(move |index| {
                                    hash_key(key_provider(index, &keys[index]))
                                })),
                            ));
                    }
                });
                Self { props: self.props }
            }

            /// Append lazy items from an iterator when only the element is used.
            pub fn items_from_iter_values<I, T, F, Output>(&self, iter: I, builder: F) -> Self
            where
                I: IntoIterator<Item = T>,
                T: Send + Sync + 'static,
                F: Fn(&T) -> Output + Send + Sync + 'static,
            {
                self.items_from_iter(iter, move |_, item| builder(item))
            }
        }
    };
}

pub(crate) use impl_lazy_builder;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct ZeroLayout;

impl tessera_ui::LayoutModifierNode for ZeroLayout {
    fn measure(
        &self,
        _input: &mut tessera_ui::layout::MeasureScope<'_>,
    ) -> Result<tessera_ui::LayoutResult, tessera_ui::MeasurementError> {
        Ok(tessera_ui::LayoutResult::new(
            tessera_ui::ComputedData::ZERO,
        ))
    }
}

#[derive(Default, Clone, PartialEq)]
pub(crate) struct FenwickTree {
    data: Vec<i64>,
}

impl FenwickTree {
    pub(crate) fn resize(&mut self, len: usize) {
        self.data.clear();
        self.data.resize(len + 1, 0);
    }

    pub(crate) fn update(&mut self, index: usize, delta: Px) {
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

    pub(crate) fn prefix_sum(&self, count: usize) -> Px {
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
