//! Nested scrolling through attached layout ancestry.
//!
//! ## Usage
//!
//! Coordinate scroll and fling consumption between nested containers.

use crate::{CallbackWith, LocalToWindow, PxPosition, ScrollEventSource, State};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Scroll displacement along each axis in the connection coordinate space.
pub struct ScrollDelta {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl ScrollDelta {
    /// Zero motion on both axes.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates an empty connection or motion value.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn clamp_consumed(self, consumed: Self) -> Self {
        Self {
            x: clamp_axis_consumed(self.x, consumed.x),
            y: clamp_axis_consumed(self.y, consumed.y),
        }
    }

    /// Returns whether both axes have negligible motion.
    pub fn is_zero(self) -> bool {
        self.x.abs() <= f32::EPSILON && self.y.abs() <= f32::EPSILON
    }
}

impl std::ops::Add for ScrollDelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for ScrollDelta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Scroll velocity along each axis in units per second.
pub struct ScrollVelocity {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl ScrollVelocity {
    /// Zero motion on both axes.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates an empty connection or motion value.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn clamp_consumed(self, consumed: Self) -> Self {
        Self {
            x: clamp_axis_consumed(self.x, consumed.x),
            y: clamp_axis_consumed(self.y, consumed.y),
        }
    }

    /// Returns whether both axes have negligible motion.
    pub fn is_zero(self) -> bool {
        self.x.abs() <= f32::EPSILON && self.y.abs() <= f32::EPSILON
    }
}

impl std::ops::Add for ScrollVelocity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for ScrollVelocity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy)]
/// Available displacement before a descendant consumes a scroll sample.
pub struct PreScrollInput {
    /// Motion available for this connection to consume.
    pub available: ScrollDelta,
    /// The input source that produced this sample.
    pub source: ScrollEventSource,
}

#[derive(Clone, Copy)]
/// Consumed and remaining displacement after a descendant scrolls.
pub struct PostScrollInput {
    /// Motion already consumed by the descendant.
    pub consumed_by_child: ScrollDelta,
    /// Motion available for this connection to consume.
    pub available: ScrollDelta,
    /// The input source that produced this sample.
    pub source: ScrollEventSource,
}

#[derive(Clone, Copy)]
/// Available velocity before a descendant begins its fling.
pub struct PreFlingInput {
    /// Motion available for this connection to consume.
    pub available: ScrollVelocity,
}

#[derive(Clone, PartialEq, Eq)]
/// Scroll and fling consumption coordinated with an ancestor connection.
pub struct NestedScrollConnection {
    coordinate_scale: Option<State<f32>>,
    parent: Option<Box<NestedScrollConnection>>,
    on_pre_scroll: CallbackWith<PreScrollInput, ScrollDelta>,
    on_post_scroll: CallbackWith<PostScrollInput, ScrollDelta>,
    on_pre_fling: CallbackWith<PreFlingInput, ScrollVelocity>,
    on_post_fling: CallbackWith<ScrollVelocity, ScrollVelocity>,
}

impl NestedScrollConnection {
    /// Creates an empty connection or motion value.
    pub fn new() -> Self {
        Self {
            coordinate_scale: None,
            parent: None,
            on_pre_scroll: CallbackWith::default_value(),
            on_post_scroll: CallbackWith::default_value(),
            on_pre_fling: CallbackWith::default_value(),
            on_post_fling: CallbackWith::default_value(),
        }
    }

    /// Uses the observed local-to-window scale for coordinate conversion.
    pub fn with_coordinate_scale(mut self, scale: State<f32>) -> Self {
        self.coordinate_scale = Some(scale);
        self
    }

    fn transform(&self) -> LocalToWindow {
        LocalToWindow::new(
            self.coordinate_scale.map_or(1.0, |scale| scale.get()),
            PxPosition::ZERO,
        )
        .expect("Nested scroll scale must be finite and positive")
    }

    /// Views this connection in the supplied caller coordinate space.
    pub fn in_space(&self, transform: LocalToWindow) -> NestedScrollSpace<'_> {
        NestedScrollSpace {
            connection: self,
            transform,
        }
    }

    /// Sets the ancestor that receives motion remaining after local consumption.
    pub fn with_parent(mut self, parent: Option<Self>) -> Self {
        self.parent = parent.map(Box::new);
        self
    }

    /// Sets the callback that consumes displacement before the child scrolls.
    pub fn with_pre_scroll_handler(
        mut self,
        handler: CallbackWith<PreScrollInput, ScrollDelta>,
    ) -> Self {
        self.on_pre_scroll = handler;
        self
    }

    /// Sets the callback that consumes remaining displacement after the child scrolls.
    pub fn with_post_scroll_handler(
        mut self,
        handler: CallbackWith<PostScrollInput, ScrollDelta>,
    ) -> Self {
        self.on_post_scroll = handler;
        self
    }

    /// Sets the callback that consumes velocity before the child flings.
    pub fn with_pre_fling_handler(
        mut self,
        handler: CallbackWith<PreFlingInput, ScrollVelocity>,
    ) -> Self {
        self.on_pre_fling = handler;
        self
    }

    /// Dispatches displacement before child consumption and returns the consumed amount.
    pub fn pre_scroll(&self, available: ScrollDelta, source: ScrollEventSource) -> ScrollDelta {
        let local = available.clamp_consumed(map_delta(
            self.transform(),
            self.on_pre_scroll.call(PreScrollInput {
                available: unmap_delta(self.transform(), available),
                source,
            }),
        ));
        let remaining = available - local;
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.pre_scroll(remaining, source))
            .unwrap_or(ScrollDelta::ZERO);
        local + parent
    }

    /// Sets the callback that consumes velocity after the child flings.
    pub fn with_post_fling_handler(
        mut self,
        handler: CallbackWith<ScrollVelocity, ScrollVelocity>,
    ) -> Self {
        self.on_post_fling = handler;
        self
    }

    /// Dispatches velocity after the child flings and returns the consumed amount.
    pub fn post_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        let local = available.clamp_consumed(map_velocity(
            self.transform(),
            self.on_post_fling
                .call(unmap_velocity(self.transform(), available)),
        ));
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.post_fling(available - local))
            .unwrap_or(ScrollVelocity::ZERO);
        local + parent
    }

    /// Dispatches remaining displacement after child consumption and returns the consumed amount.
    pub fn post_scroll(
        &self,
        consumed_by_child: ScrollDelta,
        available: ScrollDelta,
        source: ScrollEventSource,
    ) -> ScrollDelta {
        let local = available.clamp_consumed(map_delta(
            self.transform(),
            self.on_post_scroll.call(PostScrollInput {
                consumed_by_child: unmap_delta(self.transform(), consumed_by_child),
                available: unmap_delta(self.transform(), available),
                source,
            }),
        ));
        let remaining = available - local;
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.post_scroll(consumed_by_child + local, remaining, source))
            .unwrap_or(ScrollDelta::ZERO);
        local + parent
    }

    /// Dispatches velocity before the child flings and returns the consumed amount.
    pub fn pre_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        let local = available.clamp_consumed(map_velocity(
            self.transform(),
            self.on_pre_fling.call(PreFlingInput {
                available: unmap_velocity(self.transform(), available),
            }),
        ));
        let remaining = available - local;
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.pre_fling(remaining))
            .unwrap_or(ScrollVelocity::ZERO);
        local + parent
    }
}

impl Default for NestedScrollConnection {
    fn default() -> Self {
        Self::new()
    }
}

fn clamp_axis_consumed(available: f32, consumed: f32) -> f32 {
    if available > 0.0 {
        consumed.clamp(0.0, available)
    } else if available < 0.0 {
        consumed.clamp(available, 0.0)
    } else {
        0.0
    }
}

fn map_delta(transform: LocalToWindow, value: ScrollDelta) -> ScrollDelta {
    let value = transform.map_vector(PxPosition::from_f32_xy(value.x, value.y));
    ScrollDelta::new(value.x, value.y)
}
fn unmap_delta(transform: LocalToWindow, value: ScrollDelta) -> ScrollDelta {
    let value = transform.inverse_vector(PxPosition::from_f32_xy(value.x, value.y));
    ScrollDelta::new(value.x, value.y)
}
fn map_velocity(transform: LocalToWindow, value: ScrollVelocity) -> ScrollVelocity {
    let value = transform.map_vector(PxPosition::from_f32_xy(value.x, value.y));
    ScrollVelocity::new(value.x, value.y)
}
fn unmap_velocity(transform: LocalToWindow, value: ScrollVelocity) -> ScrollVelocity {
    let value = transform.inverse_vector(PxPosition::from_f32_xy(value.x, value.y));
    ScrollVelocity::new(value.x, value.y)
}

/// A connection viewed from a caller coordinate space.
pub struct NestedScrollSpace<'a> {
    connection: &'a NestedScrollConnection,
    transform: LocalToWindow,
}
impl NestedScrollSpace<'_> {
    /// Dispatches displacement before child consumption and returns the consumed amount.
    pub fn pre_scroll(&self, available: ScrollDelta, source: ScrollEventSource) -> ScrollDelta {
        unmap_delta(
            self.transform,
            self.connection
                .pre_scroll(map_delta(self.transform, available), source),
        )
    }
    /// Dispatches remaining displacement after child consumption and returns the consumed amount.
    pub fn post_scroll(
        &self,
        consumed: ScrollDelta,
        available: ScrollDelta,
        source: ScrollEventSource,
    ) -> ScrollDelta {
        unmap_delta(
            self.transform,
            self.connection.post_scroll(
                map_delta(self.transform, consumed),
                map_delta(self.transform, available),
                source,
            ),
        )
    }
    /// Dispatches velocity before the child flings and returns the consumed amount.
    pub fn pre_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        unmap_velocity(
            self.transform,
            self.connection
                .pre_fling(map_velocity(self.transform, available)),
        )
    }
    /// Dispatches velocity after the child flings and returns the consumed amount.
    pub fn post_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        unmap_velocity(
            self.transform,
            self.connection
                .post_fling(map_velocity(self.transform, available)),
        )
    }
}

#[derive(Clone)]
struct ScrollReader {
    anchor_instance: u64,
    provider: Option<u64>,
    connection: Option<NestedScrollConnection>,
}

#[derive(Default)]
struct ScrollDependencies {
    readers: rustc_hash::FxHashMap<u64, ScrollReader>,
    by_provider: rustc_hash::FxHashMap<Option<u64>, rustc_hash::FxHashSet<u64>>,
    owners: rustc_hash::FxHashMap<u64, Option<NestedScrollConnection>>,
    pending_readers: rustc_hash::FxHashSet<u64>,
}

#[derive(Default)]
struct ScrollObserver(std::cell::RefCell<ScrollDependencies>);

fn nearest_above(anchor: crate::NodeId) -> (Option<u64>, Option<NestedScrollConnection>) {
    crate::runtime::TesseraRuntime::with(|runtime| {
        let tree = runtime.component_tree.tree();
        let mut parent = tree.get(anchor).and_then(|node| node.parent());
        while let Some(id) = parent {
            let Some(node) = tree.get(id) else { break };
            if let Some(connection) = node.get().modifier.nested_scroll_connection() {
                return (Some(node.get().instance_logic_id), Some(connection));
            }
            parent = node.parent();
        }
        (None, None)
    })
}

impl NestedScrollConnection {
    /// Returns the nearest connection attached above the current build node.
    pub fn nearest() -> Option<Self> {
        crate::runtime::ensure_build_phase();
        let (anchor, anchor_instance) = crate::runtime::TesseraRuntime::with(|runtime| {
            let anchor = runtime.component_tree.current_build_node_id()?;
            let instance = runtime
                .component_tree
                .tree()
                .get(anchor)?
                .get()
                .instance_key;
            Some((anchor, instance))
        })?;
        let (provider, connection) = nearest_above(anchor);
        if let Some(boundary) = crate::runtime::current_replay_boundary_instance_key_from_scope() {
            let observer = crate::local_runtime_resource::<ScrollObserver>();
            let previous = {
                let mut dependencies = observer.0.borrow_mut();
                if let Some(previous) = dependencies.readers.get(&boundary) {
                    let previous_provider = previous.provider;
                    if let Some(readers) = dependencies.by_provider.get_mut(&previous_provider) {
                        readers.remove(&boundary);
                    }
                }
                dependencies
                    .by_provider
                    .entry(provider)
                    .or_default()
                    .insert(boundary);
                dependencies.readers.insert(
                    boundary,
                    ScrollReader {
                        anchor_instance,
                        provider,
                        connection: connection.clone(),
                    },
                )
            };
            drop(previous);
        }
        connection
    }
}

pub(crate) fn attachment_changed() {
    let Some((anchor, owner, connection)) = crate::runtime::TesseraRuntime::with(|runtime| {
        let anchor = runtime.component_tree.current_build_node_id()?;
        let node = runtime.component_tree.current_node()?;
        Some((
            anchor,
            node.instance_logic_id,
            node.modifier.nested_scroll_connection(),
        ))
    }) else {
        return;
    };
    let observer = crate::local_runtime_resource::<ScrollObserver>();
    let previous = {
        let mut dependencies = observer.0.borrow_mut();
        if dependencies
            .owners
            .get(&owner)
            .is_some_and(|previous| previous == &connection)
            || (connection.is_none() && !dependencies.owners.contains_key(&owner))
        {
            return;
        }
        let previous = dependencies.owners.insert(owner, connection);
        let parent_provider = nearest_above(anchor).0;
        for provider in [Some(owner), parent_provider] {
            if let Some(readers) = dependencies.by_provider.get(&provider) {
                let readers = readers.iter().copied().collect::<Vec<_>>();
                dependencies.pending_readers.extend(readers);
            }
        }
        previous
    };
    drop(previous);
}

pub(crate) fn flush_attachment_changes() {
    let observer = crate::local_runtime_resource::<ScrollObserver>();
    let candidates = {
        let mut dependencies = observer.0.borrow_mut();
        std::mem::take(&mut dependencies.pending_readers)
            .into_iter()
            .filter_map(|reader| {
                dependencies
                    .readers
                    .get(&reader)
                    .cloned()
                    .map(|dependency| (reader, dependency))
            })
            .collect::<Vec<_>>()
    };
    for (reader, dependency) in candidates {
        let anchor = crate::runtime::TesseraRuntime::with(|runtime| {
            runtime
                .component_tree
                .find_node_id_by_instance_key(dependency.anchor_instance)
        });
        let Some(anchor) = anchor else {
            continue;
        };
        let current = nearest_above(anchor);
        if current.0 != dependency.provider || current.1 != dependency.connection {
            crate::runtime::record_replay_boundary_invalidation_for_instance_key(reader);
        }
    }
}

pub(crate) fn remove_readers(keys: &rustc_hash::FxHashSet<u64>) {
    let observer = crate::local_runtime_resource::<ScrollObserver>();
    let removed = {
        let mut dependencies = observer.0.borrow_mut();
        let mut removed = Vec::new();
        for key in keys {
            dependencies.pending_readers.remove(key);
            if let Some(reader) = dependencies.readers.remove(key) {
                if let Some(readers) = dependencies.by_provider.get_mut(&reader.provider) {
                    readers.remove(key);
                    if readers.is_empty() {
                        dependencies.by_provider.remove(&reader.provider);
                    }
                }
                removed.push(reader);
            }
        }
        removed
    };
    drop(removed);
}

pub(crate) fn remove_owners(keys: &rustc_hash::FxHashSet<u64>) {
    let observer = crate::local_runtime_resource::<ScrollObserver>();
    let removed: Vec<_> = {
        let mut dependencies = observer.0.borrow_mut();
        keys.iter()
            .filter_map(|key| dependencies.owners.remove(key))
            .collect()
    };
    drop(removed);
}

impl crate::modifier::BuildModifierNode for NestedScrollConnection {
    fn apply(&self, _runtime: &mut crate::runtime::TesseraRuntime) {}

    fn nested_scroll_connection(&self) -> Option<Self> {
        Some(self.clone())
    }
}
