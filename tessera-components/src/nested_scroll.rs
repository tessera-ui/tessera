use tessera_ui::{CallbackWith, ScrollEventSource};

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
    parent: Option<Box<NestedScrollConnection>>,
    on_pre_scroll: CallbackWith<PreScrollInput, ScrollDelta>,
    on_post_scroll: CallbackWith<PostScrollInput, ScrollDelta>,
    on_pre_fling: CallbackWith<PreFlingInput, ScrollVelocity>,
}

impl NestedScrollConnection {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            on_pre_scroll: CallbackWith::default_value(),
            on_post_scroll: CallbackWith::default_value(),
            on_pre_fling: CallbackWith::default_value(),
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
        let local = available.clamp_consumed(
            self.on_pre_scroll
                .call(PreScrollInput { available, source }),
        );
        let remaining = available - local;
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.pre_scroll(remaining, source))
            .unwrap_or(ScrollDelta::ZERO);
        local + parent
    }

    pub(crate) fn post_scroll(
        &self,
        consumed_by_child: ScrollDelta,
        available: ScrollDelta,
        source: ScrollEventSource,
    ) -> ScrollDelta {
        let local = available.clamp_consumed(self.on_post_scroll.call(PostScrollInput {
            consumed_by_child,
            available,
            source,
        }));
        let remaining = available - local;
        let parent = self
            .parent
            .as_ref()
            .map(|connection| connection.post_scroll(consumed_by_child + local, remaining, source))
            .unwrap_or(ScrollDelta::ZERO);
        local + parent
    }

    pub(crate) fn pre_fling(&self, available: ScrollVelocity) -> ScrollVelocity {
        let local = available.clamp_consumed(self.on_pre_fling.call(PreFlingInput { available }));
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    };
    use tessera_ui::{CallbackWith, ScrollEventSource};

    #[test]
    fn pre_scroll_clamps_consumption_to_available_delta() {
        let connection = NestedScrollConnection::new()
            .with_pre_scroll_handler(CallbackWith::new(|_| ScrollDelta::new(0.0, 100.0)));

        let consumed = connection.pre_scroll(ScrollDelta::new(0.0, 24.0), ScrollEventSource::Touch);

        assert_eq!(consumed, ScrollDelta::new(0.0, 24.0));
    }

    #[test]
    fn pre_scroll_chains_parent_with_remaining_delta() {
        let child_inputs = Arc::new(Mutex::new(Vec::new()));
        let parent_inputs = Arc::new(Mutex::new(Vec::new()));
        let parent = NestedScrollConnection::new().with_pre_scroll_handler({
            let parent_inputs = Arc::clone(&parent_inputs);
            CallbackWith::new(move |input: PreScrollInput| {
                parent_inputs
                    .lock()
                    .expect("parent inputs lock should not be poisoned")
                    .push(input.available);
                ScrollDelta::new(0.0, 5.0)
            })
        });
        let connection = NestedScrollConnection::new()
            .with_pre_scroll_handler({
                let child_inputs = Arc::clone(&child_inputs);
                CallbackWith::new(move |input: PreScrollInput| {
                    child_inputs
                        .lock()
                        .expect("child inputs lock should not be poisoned")
                        .push(input.available);
                    ScrollDelta::new(0.0, 10.0)
                })
            })
            .with_parent(Some(parent));

        let consumed = connection.pre_scroll(ScrollDelta::new(0.0, 30.0), ScrollEventSource::Touch);

        assert_eq!(consumed, ScrollDelta::new(0.0, 15.0));
        assert_eq!(
            child_inputs
                .lock()
                .expect("child inputs lock should not be poisoned")
                .as_slice(),
            &[ScrollDelta::new(0.0, 30.0)]
        );
        assert_eq!(
            parent_inputs
                .lock()
                .expect("parent inputs lock should not be poisoned")
                .as_slice(),
            &[ScrollDelta::new(0.0, 20.0)]
        );
    }

    #[test]
    fn post_scroll_passes_accumulated_consumed_delta_to_parent() {
        let parent_inputs = Arc::new(Mutex::new(Vec::new()));
        let parent = NestedScrollConnection::new().with_post_scroll_handler({
            let parent_inputs = Arc::clone(&parent_inputs);
            CallbackWith::new(move |input: PostScrollInput| {
                parent_inputs
                    .lock()
                    .expect("parent inputs lock should not be poisoned")
                    .push((input.consumed_by_child, input.available));
                ScrollDelta::new(0.0, 4.0)
            })
        });
        let connection = NestedScrollConnection::new()
            .with_post_scroll_handler(CallbackWith::new(|_| ScrollDelta::new(0.0, 6.0)))
            .with_parent(Some(parent));

        let consumed = connection.post_scroll(
            ScrollDelta::new(0.0, 20.0),
            ScrollDelta::new(0.0, 30.0),
            ScrollEventSource::Touch,
        );

        assert_eq!(consumed, ScrollDelta::new(0.0, 10.0));
        assert_eq!(
            parent_inputs
                .lock()
                .expect("parent inputs lock should not be poisoned")
                .as_slice(),
            &[(ScrollDelta::new(0.0, 26.0), ScrollDelta::new(0.0, 24.0))]
        );
    }

    #[test]
    fn pre_fling_chains_parent_with_remaining_velocity() {
        let parent_inputs = Arc::new(Mutex::new(Vec::new()));
        let parent = NestedScrollConnection::new().with_pre_fling_handler({
            let parent_inputs = Arc::clone(&parent_inputs);
            CallbackWith::new(move |input: PreFlingInput| {
                parent_inputs
                    .lock()
                    .expect("parent inputs lock should not be poisoned")
                    .push(input.available);
                ScrollVelocity::new(0.0, 8.0)
            })
        });
        let connection = NestedScrollConnection::new()
            .with_pre_fling_handler(CallbackWith::new(|_| ScrollVelocity::new(0.0, 12.0)))
            .with_parent(Some(parent));

        let consumed = connection.pre_fling(ScrollVelocity::new(0.0, 30.0));

        assert_eq!(consumed, ScrollVelocity::new(0.0, 20.0));
        assert_eq!(
            parent_inputs
                .lock()
                .expect("parent inputs lock should not be poisoned")
                .as_slice(),
            &[ScrollVelocity::new(0.0, 18.0)]
        );
    }
}
