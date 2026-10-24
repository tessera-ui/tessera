use tessera_ui::{CallbackWith, LocalToWindow, PxPosition, ScrollEventSource, State};

#[cfg(test)]
#[path = "nested_scroll_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScrollDelta {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl ScrollDelta {
    pub(crate) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn clamp_consumed(self, consumed: Self) -> Self {
        Self {
            x: clamp_axis_consumed(self.x, consumed.x),
            y: clamp_axis_consumed(self.y, consumed.y),
        }
    }

    pub(crate) fn is_zero(self) -> bool {
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
pub(crate) struct ScrollVelocity {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl ScrollVelocity {
    pub(crate) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn clamp_consumed(self, consumed: Self) -> Self {
        Self {
            x: clamp_axis_consumed(self.x, consumed.x),
            y: clamp_axis_consumed(self.y, consumed.y),
        }
    }

    pub(crate) fn is_zero(self) -> bool {
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
pub(crate) struct PreScrollInput {
    pub(crate) available: ScrollDelta,
    pub(crate) source: ScrollEventSource,
}

#[derive(Clone, Copy)]
pub(crate) struct PostScrollInput {
    pub(crate) consumed_by_child: ScrollDelta,
    pub(crate) available: ScrollDelta,
    pub(crate) source: ScrollEventSource,
}

#[derive(Clone, Copy)]
pub(crate) struct PreFlingInput {
    pub(crate) available: ScrollVelocity,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct NestedScrollConnection {
    coordinate_scale: Option<State<f32>>,
    parent: Option<Box<NestedScrollConnection>>,
    on_pre_scroll: CallbackWith<PreScrollInput, ScrollDelta>,
    on_post_scroll: CallbackWith<PostScrollInput, ScrollDelta>,
    on_pre_fling: CallbackWith<PreFlingInput, ScrollVelocity>,
    on_post_fling: CallbackWith<ScrollVelocity, ScrollVelocity>,
}

impl NestedScrollConnection {
    pub(crate) fn new() -> Self {
        Self {
            coordinate_scale: None,
            parent: None,
            on_pre_scroll: CallbackWith::default_value(),
            on_post_scroll: CallbackWith::default_value(),
            on_pre_fling: CallbackWith::default_value(),
            on_post_fling: CallbackWith::default_value(),
        }
    }

    pub(crate) fn with_coordinate_scale(mut self, scale: State<f32>) -> Self {
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

    pub(crate) fn in_space(&self, transform: LocalToWindow) -> NestedScrollSpace<'_> {
        NestedScrollSpace {
            connection: self,
            transform,
        }
    }

    pub(crate) fn with_parent(mut self, parent: Option<Self>) -> Self {
        self.parent = parent.map(Box::new);
        self
    }

    pub(crate) fn with_pre_scroll_handler(
        mut self,
        handler: CallbackWith<PreScrollInput, ScrollDelta>,
    ) -> Self {
        self.on_pre_scroll = handler;
        self
    }

    pub(crate) fn with_post_scroll_handler(
        mut self,
        handler: CallbackWith<PostScrollInput, ScrollDelta>,
    ) -> Self {
        self.on_post_scroll = handler;
        self
    }

    pub(crate) fn with_pre_fling_handler(
        mut self,
        handler: CallbackWith<PreFlingInput, ScrollVelocity>,
    ) -> Self {
        self.on_pre_fling = handler;
        self
    }

    pub(crate) fn pre_scroll(
        &self,
        available: ScrollDelta,
        source: ScrollEventSource,
    ) -> ScrollDelta {
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

    pub(crate) fn with_post_fling_handler(
        mut self,
        handler: CallbackWith<ScrollVelocity, ScrollVelocity>,
    ) -> Self {
        self.on_post_fling = handler;
        self
    }

    pub(crate) fn post_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
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

    pub(crate) fn post_scroll(
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

    pub(crate) fn pre_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
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

pub(crate) struct NestedScrollSpace<'a> {
    connection: &'a NestedScrollConnection,
    transform: LocalToWindow,
}
impl NestedScrollSpace<'_> {
    pub(crate) fn pre_scroll(
        &self,
        available: ScrollDelta,
        source: ScrollEventSource,
    ) -> ScrollDelta {
        unmap_delta(
            self.transform,
            self.connection
                .pre_scroll(map_delta(self.transform, available), source),
        )
    }
    pub(crate) fn post_scroll(
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
    pub(crate) fn pre_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        unmap_velocity(
            self.transform,
            self.connection
                .pre_fling(map_velocity(self.transform, available)),
        )
    }
    pub(crate) fn post_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        unmap_velocity(
            self.transform,
            self.connection
                .post_fling(map_velocity(self.transform, available)),
        )
    }
}
