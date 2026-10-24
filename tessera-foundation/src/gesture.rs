//! Pointer gesture recognizers for tap, drag, long-press, and scroll.
//!
//! ## Usage
//!
//! Use recognizers to derive reusable pointer interaction behavior in shared
//! modifiers and components.

use std::time::Duration;

use tessera_ui::{
    CursorEventContent, GestureState, PointerChange, PointerEventPass, PointerId,
    PressKeyEventType, Px, PxPosition, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    time::Instant,
};

const DEFAULT_SLOP_PX: f32 = 8.0;

/// Shared bookkeeping for a single press across pointer updates.
///
/// Recorded press positions anchor gesture travel. Synthetic events without
/// a position fall back to the first movement sample.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct PressTrack {
    /// Recorded press position, or the first available movement sample.
    start: Option<PxPosition>,
    /// Most recent movement sample, used as the delta baseline.
    last: Option<PxPosition>,
}

impl PressTrack {
    fn on_press(&mut self, position: Option<PxPosition>) {
        self.start = position;
        self.last = position;
    }

    /// Folds one movement sample into the track and reports the derived facts.
    ///
    /// `slop_px` is the slop radius used to compute
    /// `Observation::exceeded_slop`. The first sample after a press
    /// (`Observation::started_now`) only establishes the origin, so it reports
    /// no travel, no delta, and cannot have exceeded the slop yet.
    fn observe(&mut self, position: PxPosition, slop_px: f32) -> Observation {
        let Some(start) = self.start else {
            self.start = Some(position);
            self.last = Some(position);
            return Observation {
                started_now: true,
                exceeded_slop: false,
                travel: (0.0, 0.0),
                delta: (Px::ZERO, Px::ZERO),
            };
        };
        let delta = match self.last {
            Some(last) => (
                Px((position.x - last.x).round() as i32),
                Px((position.y - last.y).round() as i32),
            ),
            None => (Px::ZERO, Px::ZERO),
        };
        self.last = Some(position);
        Observation {
            started_now: false,
            exceeded_slop: start.distance_to(position) > slop_px,
            travel: (position.x - start.x, position.y - start.y),
            delta,
        }
    }
}

/// Facts derived from one movement sample relative to the tracked press.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Observation {
    /// `true` when this sample established the slop origin, i.e. it is the
    /// first movement sample observed after the press.
    started_now: bool,
    /// `true` when the sample lies strictly farther than the slop radius from
    /// the origin. Always `false` for the establishing sample.
    exceeded_slop: bool,
    /// Travel from the origin along each axis, in pixels.
    travel: (f32, f32),
    /// Movement since the previous sample, in pixels.
    delta: (Px, Px),
}

/// Configuration for tap gesture recognition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapSettings {
    /// Mouse button or press key that starts the gesture.
    pub button: PressKeyEventType,
    /// Maximum pointer travel before the tap is canceled.
    pub slop_px: f32,
    /// Whether to consume the press event immediately.
    pub consume_on_press: bool,
    /// Whether to consume the release event.
    pub consume_on_release: bool,
    /// Whether to consume the completed tap event.
    pub consume_on_tap: bool,
}

impl Default for TapSettings {
    fn default() -> Self {
        Self {
            button: PressKeyEventType::Left,
            slop_px: DEFAULT_SLOP_PX,
            consume_on_press: false,
            consume_on_release: false,
            consume_on_tap: false,
        }
    }
}

/// Per-update tap recognition output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TapResult {
    /// Whether a press started this update.
    pub pressed: bool,
    /// Whether a release happened this update.
    pub released: bool,
    /// Whether a full tap gesture completed this update.
    pub tapped: bool,
    /// Timestamp for the press, when present.
    pub press_timestamp: Option<Instant>,
    /// Timestamp for the release, when present.
    pub release_timestamp: Option<Instant>,
}

/// Stateful tap gesture recognizer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapRecognizer {
    settings: TapSettings,
    active_pointer: Option<PointerId>,
    track: PressTrack,
    canceled: bool,
}

impl TapRecognizer {
    /// Creates a tap recognizer with custom settings.
    pub fn new(settings: TapSettings) -> Self {
        Self {
            settings,
            active_pointer: None,
            track: PressTrack::default(),
            canceled: false,
        }
    }

    /// Updates the recognizer with the current pointer pass and events.
    ///
    /// # Position source
    ///
    /// Positions come from the samples themselves in a shared coordinate space.
    ///
    /// # Slop origin
    ///
    /// The recorded press position anchors slop and movement. A press without
    /// a recorded position uses the first movement sample as its origin.
    ///
    /// # `within_bounds`
    ///
    /// `within_bounds` is a semantic flag the caller computes from the node's
    /// relative coordinates; it reports whether the pointer was inside this
    /// node when the update ran.
    pub fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
    ) -> TapResult {
        self.update_inner(pass, pointer_changes, within_bounds, false)
    }

    /// Recognizes taps while claiming each press before ancestor tap handlers.
    ///
    /// Use for nested click targets. Claims preserve samples for drag and
    /// scroll observers that use [`Self::update`].
    pub fn update_exclusive(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
    ) -> TapResult {
        self.update_inner(pass, pointer_changes, within_bounds, true)
    }

    fn update_inner(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
        exclusive: bool,
    ) -> TapResult {
        if pass != PointerEventPass::Main {
            return TapResult::default();
        }

        let mut result = TapResult::default();
        for change in pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            match change.content {
                CursorEventContent::Pressed(button)
                    if button == self.settings.button && within_bounds =>
                {
                    if exclusive && !change.claim_tap_press() {
                        continue;
                    }
                    self.active_pointer = Some(change.pointer_id);
                    self.track.on_press(change.position());
                    self.canceled = false;
                    result.pressed = true;
                    result.press_timestamp = Some(change.timestamp);
                    if self.settings.consume_on_press {
                        change.consume();
                    }
                }
                CursorEventContent::Moved(position) => {
                    if Some(change.pointer_id) == self.active_pointer {
                        let observation = self.track.observe(position, self.settings.slop_px);
                        if observation.exceeded_slop {
                            self.canceled = true;
                        }
                    }
                }
                CursorEventContent::Scroll(_) if Some(change.pointer_id) == self.active_pointer => {
                    self.canceled = true;
                }
                CursorEventContent::Cancelled(button) if button == self.settings.button => {
                    if Some(change.pointer_id) == self.active_pointer {
                        result.released = true;
                        self.reset();
                    }
                }
                CursorEventContent::Released(button) if button == self.settings.button => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    result.released = true;
                    result.release_timestamp = Some(change.timestamp);
                    let tapped = within_bounds
                        && !self.canceled
                        && change.gesture_state == GestureState::TapCandidate;
                    if tapped {
                        result.tapped = true;
                    }
                    if self.settings.consume_on_release || (self.settings.consume_on_tap && tapped)
                    {
                        change.consume();
                    }
                    self.reset();
                }
                _ => {}
            }
        }
        result
    }

    /// Cancels the active press without producing a tap.
    pub fn cancel(&mut self) {
        self.reset();
    }

    fn reset(&mut self) {
        self.active_pointer = None;
        self.track.on_press(None);
        self.canceled = false;
    }
}

impl Default for TapRecognizer {
    fn default() -> Self {
        Self::new(TapSettings::default())
    }
}

/// Configuration for drag gesture recognition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragSettings {
    /// Minimum pointer travel required to start dragging.
    pub slop_px: f32,
    /// Whether drag events should be consumed after dragging starts.
    pub consume_when_dragging: bool,
    /// Optional axis lock for drag movement.
    pub axis: Option<DragAxis>,
}

impl Default for DragSettings {
    fn default() -> Self {
        Self {
            slop_px: DEFAULT_SLOP_PX,
            consume_when_dragging: true,
            axis: None,
        }
    }
}

/// Axis constraint for drag gesture recognition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragAxis {
    /// Horizontal-only dragging.
    Horizontal,
    /// Vertical-only dragging.
    Vertical,
}

/// Per-update drag recognition output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DragResult {
    /// Whether dragging started this update.
    pub started: bool,
    /// Whether a drag delta was produced this update.
    pub updated: bool,
    /// Whether dragging ended this update.
    pub ended: bool,
    /// Horizontal drag delta for this update.
    pub delta_x: Px,
    /// Vertical drag delta for this update.
    pub delta_y: Px,
}

/// Stateful drag gesture recognizer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragRecognizer {
    delta_remainder: PxPosition,
    settings: DragSettings,
    active_pointer: Option<PointerId>,
    track: PressTrack,
    dragging: bool,
}

impl DragRecognizer {
    /// Creates a drag recognizer with custom settings.
    pub fn new(settings: DragSettings) -> Self {
        Self {
            delta_remainder: PxPosition::ZERO,
            settings,
            active_pointer: None,
            track: PressTrack::default(),
            dragging: false,
        }
    }

    /// Updates the recognizer with the current pointer pass and events.
    ///
    /// # Position source
    ///
    /// Positions come only from the events themselves
    /// (`CursorEventContent::Moved`); the caller neither needs to nor can
    /// provide a coordinate space.
    ///
    /// # Slop origin
    ///
    /// The press event carries no position, so the slop origin is the **first
    /// movement sample** observed after the press. Displacement that happens
    /// entirely within the press frame is therefore not counted toward
    /// `slop_px`, bounding the error by the pointer movement of that one frame.
    ///
    /// # `within_bounds`
    ///
    /// `within_bounds` is a semantic flag the caller computes from the node's
    /// relative coordinates; it reports whether the pointer was inside this
    /// node when the update ran.
    pub fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
    ) -> DragResult {
        if pass != PointerEventPass::Main {
            return DragResult::default();
        }

        let mut result = DragResult::default();
        for change in pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) if within_bounds => {
                    self.delta_remainder = PxPosition::ZERO;
                    self.active_pointer = Some(change.pointer_id);
                    self.track.on_press(change.position());
                    self.dragging = false;
                }
                CursorEventContent::Moved(position) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    let observation = self.track.observe(position, self.settings.slop_px);
                    if observation.started_now {
                        continue;
                    }
                    if !self.dragging {
                        let (travel_x, travel_y) = observation.travel;
                        let should_start = match self.settings.axis {
                            Some(DragAxis::Horizontal) => {
                                travel_x.abs() > self.settings.slop_px
                                    && travel_x.abs() >= travel_y.abs()
                            }
                            Some(DragAxis::Vertical) => {
                                travel_y.abs() > self.settings.slop_px
                                    && travel_y.abs() >= travel_x.abs()
                            }
                            None => observation.exceeded_slop,
                        };
                        if should_start {
                            self.dragging = true;
                            result.started = true;
                        }
                    }
                    if self.dragging {
                        let (raw_dx, raw_dy) = observation.delta;
                        let (dx, dy) = match self.settings.axis {
                            Some(DragAxis::Horizontal) => (raw_dx, Px::ZERO),
                            Some(DragAxis::Vertical) => (Px::ZERO, raw_dy),
                            None => (raw_dx, raw_dy),
                        };
                        if dx != Px::ZERO || dy != Px::ZERO {
                            result.delta_x += dx;
                            result.delta_y += dy;
                            result.updated = true;
                        }
                        if self.settings.consume_when_dragging {
                            change.consume();
                        }
                    }
                }
                CursorEventContent::Cancelled(PressKeyEventType::Left) => {
                    if Some(change.pointer_id) == self.active_pointer {
                        self.reset();
                    }
                }
                CursorEventContent::Released(PressKeyEventType::Left) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    result.ended = true;
                    if self.dragging && self.settings.consume_when_dragging {
                        change.consume();
                    }
                    self.reset();
                }
                CursorEventContent::Scroll(_) => {}
                _ => {}
            }
        }
        result
    }

    /// Replaces the recognizer settings while preserving active gesture state.
    pub fn set_settings(&mut self, settings: DragSettings) {
        self.settings = settings;
    }

    /// Returns whether dragging is currently active.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub(crate) fn local_integer_delta(
        &mut self,
        x: Px,
        y: Px,
        transform: tessera_ui::LocalToWindow,
    ) -> (Px, Px) {
        let window = PxPosition::new(x, y) + self.delta_remainder;
        let local = transform.inverse_vector(window);
        let rounded = (local.x_px(), local.y_px());
        self.delta_remainder = window - transform.map_vector(PxPosition::new(rounded.0, rounded.1));
        rounded
    }

    fn reset(&mut self) {
        self.active_pointer = None;
        self.track.on_press(None);
        self.dragging = false;
    }
}

impl Default for DragRecognizer {
    fn default() -> Self {
        Self::new(DragSettings::default())
    }
}

/// Configuration for long-press gesture recognition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongPressSettings {
    /// Minimum press duration before the gesture triggers.
    pub threshold: Duration,
    /// Maximum pointer travel before the press is canceled.
    pub slop_px: f32,
    /// Whether to consume the triggering event.
    pub consume_on_trigger: bool,
}

impl Default for LongPressSettings {
    fn default() -> Self {
        Self {
            threshold: Duration::from_millis(500),
            slop_px: DEFAULT_SLOP_PX,
            consume_on_trigger: false,
        }
    }
}

/// Per-update long-press recognition output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LongPressResult {
    /// Whether the long press triggered this update.
    pub triggered: bool,
    /// Whether the pointer was released this update.
    pub released: bool,
}

/// Stateful long-press gesture recognizer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongPressRecognizer {
    settings: LongPressSettings,
    active_pointer: Option<PointerId>,
    track: PressTrack,
    press_time: Option<Instant>,
    canceled: bool,
    triggered: bool,
    frame_press_start: Option<u64>,
    within_bounds: bool,
}

impl LongPressRecognizer {
    /// Creates a long-press recognizer with custom settings.
    pub fn new(settings: LongPressSettings) -> Self {
        Self {
            settings,
            active_pointer: None,
            track: PressTrack::default(),
            press_time: None,
            canceled: false,
            triggered: false,
            frame_press_start: None,
            within_bounds: false,
        }
    }

    /// Updates the recognizer with the current pointer pass and events.
    ///
    /// # Position source
    ///
    /// Positions come only from the events themselves
    /// (`CursorEventContent::Moved`); the caller neither needs to nor can
    /// provide a coordinate space.
    ///
    /// # Slop origin
    ///
    /// The press event carries no position, so the slop origin is the **first
    /// movement sample** observed after the press. Displacement that happens
    /// entirely within the press frame is therefore not counted toward
    /// `slop_px`, bounding the error by the pointer movement of that one frame.
    ///
    /// # `within_bounds`
    ///
    /// `within_bounds` is a semantic flag the caller computes from the node's
    /// relative coordinates; it reports whether the pointer was inside this
    /// node when the update ran.
    pub fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
    ) -> LongPressResult {
        self.update_inner(pass, pointer_changes, within_bounds, None, None)
    }

    /// Updates a press using the UI frame clock, including final-pass drag
    /// cancellation.
    pub fn update_at_frame_nanos(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
        frame_nanos: u64,
    ) -> LongPressResult {
        self.update_frame_inner(pass, pointer_changes, within_bounds, frame_nanos, None)
    }

    /// Observes the samples belonging to the supplied tap result, including
    /// consumed releases.
    pub fn update_after_tap(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
        frame_nanos: u64,
        tap: TapResult,
    ) -> LongPressResult {
        self.update_frame_inner(pass, pointer_changes, within_bounds, frame_nanos, Some(tap))
    }

    fn update_frame_inner(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
        frame_nanos: u64,
        tap: Option<TapResult>,
    ) -> LongPressResult {
        if pass == PointerEventPass::Final {
            if pointer_changes.iter().any(|change| {
                Some(change.pointer_id) == self.active_pointer
                    && (change.gesture_state == GestureState::Dragged
                        || matches!(change.content, CursorEventContent::Scroll(_)))
            }) {
                self.canceled = true;
            }
            return LongPressResult::default();
        }
        self.update_inner(pass, pointer_changes, within_bounds, Some(frame_nanos), tap)
    }

    /// Reports whether a frame-clock press still needs its deadline checked.
    pub fn needs_frame(&self) -> bool {
        self.active_pointer.is_some()
            && self.frame_press_start.is_some()
            && !self.canceled
            && !self.triggered
    }

    /// Reports whether the current press has already triggered.
    pub fn has_triggered(&self) -> bool {
        self.triggered
    }

    /// Reports whether a pointer press is being tracked.
    pub fn is_active(&self) -> bool {
        self.active_pointer.is_some()
    }

    /// Cancels the current press without activation.
    pub fn cancel(&mut self) {
        self.reset();
    }

    /// Advances a stationary press and returns true only at its first deadline.
    pub fn advance_frame_nanos(&mut self, frame_nanos: u64) -> bool {
        if !self.needs_frame() || !self.within_bounds {
            return false;
        }
        let elapsed = frame_nanos.saturating_sub(self.frame_press_start.unwrap());
        if u128::from(elapsed) < self.settings.threshold.as_nanos() {
            return false;
        }
        self.triggered = true;
        true
    }

    fn update_inner(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        within_bounds: bool,
        frame_nanos: Option<u64>,
        tap: Option<TapResult>,
    ) -> LongPressResult {
        if pass != PointerEventPass::Main {
            return LongPressResult::default();
        }
        let mut result = LongPressResult::default();
        for change in pointer_changes.iter_mut() {
            let owned = tap.is_some_and(|tap| match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) => tap.pressed,
                CursorEventContent::Released(PressKeyEventType::Left)
                | CursorEventContent::Cancelled(PressKeyEventType::Left) => tap.released,
                _ => false,
            });
            if change.is_consumed() && !owned {
                continue;
            }
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    if within_bounds && tap.is_none_or(|tap| tap.pressed) =>
                {
                    self.active_pointer = Some(change.pointer_id);
                    self.within_bounds = within_bounds;
                    self.track.on_press(change.position());
                    self.press_time = Some(change.timestamp);
                    self.frame_press_start = frame_nanos;
                    self.canceled = false;
                    self.triggered = false;
                }
                CursorEventContent::Moved(position) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    self.within_bounds = within_bounds;
                    let observation = self.track.observe(position, self.settings.slop_px);
                    if observation.exceeded_slop || (frame_nanos.is_some() && !within_bounds) {
                        self.canceled = true;
                    }
                    self.try_trigger(
                        change.timestamp,
                        within_bounds,
                        frame_nanos,
                        change,
                        &mut result,
                    );
                }
                CursorEventContent::Scroll(_) if Some(change.pointer_id) == self.active_pointer => {
                    self.canceled = true;
                }
                CursorEventContent::Cancelled(PressKeyEventType::Left) => {
                    if Some(change.pointer_id) == self.active_pointer {
                        self.reset();
                    }
                }
                CursorEventContent::Released(PressKeyEventType::Left) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    self.within_bounds = within_bounds;
                    self.try_trigger(
                        change.timestamp,
                        within_bounds,
                        frame_nanos,
                        change,
                        &mut result,
                    );
                    result.released = true;
                    self.reset();
                }
                _ => {}
            }
        }
        result
    }

    fn try_trigger(
        &mut self,
        now: Instant,
        within_bounds: bool,
        frame_nanos: Option<u64>,
        change: &mut PointerChange,
        result: &mut LongPressResult,
    ) {
        if let Some(frame) = frame_nanos {
            if self.advance_frame_nanos(frame) {
                result.triggered = true;
                if self.settings.consume_on_trigger {
                    change.consume();
                }
                return;
            }
        }
        if self.triggered || self.canceled || !within_bounds {
            return;
        }
        let Some(start) = self.press_time else {
            return;
        };
        if now.duration_since(start) >= self.settings.threshold {
            self.triggered = true;
            result.triggered = true;
            if self.settings.consume_on_trigger {
                change.consume();
            }
        }
    }

    fn reset(&mut self) {
        self.active_pointer = None;
        self.track.on_press(None);
        self.press_time = None;
        self.frame_press_start = None;
        self.within_bounds = false;
        self.canceled = false;
        self.triggered = false;
    }
}

impl Default for LongPressRecognizer {
    fn default() -> Self {
        Self::new(LongPressSettings::default())
    }
}

/// Configuration for scroll gesture aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ScrollSettings {
    /// Whether to consume scroll events after they are aggregated.
    pub consume: bool,
}

/// Aggregated scroll recognition output.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollResult {
    /// Number of scroll events observed in this update.
    pub event_count: usize,
    /// Total horizontal scroll delta.
    pub delta_x: f32,
    /// Total vertical scroll delta.
    pub delta_y: f32,
    /// Source of the first observed scroll event.
    pub source: Option<ScrollEventSource>,
    /// Delta unit of the first observed scroll event.
    pub unit: Option<ScrollDeltaUnit>,
}

impl Default for ScrollResult {
    fn default() -> Self {
        Self {
            event_count: 0,
            delta_x: 0.0,
            delta_y: 0.0,
            source: None,
            unit: None,
        }
    }
}

impl ScrollResult {
    /// Returns whether any scroll event was observed in this update.
    pub fn has_scroll(&self) -> bool {
        self.event_count > 0
    }
}

/// Metadata for a single scroll event routed through
/// `ScrollRecognizer::for_each`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollChangeContext {
    /// Pointer id that produced the scroll event.
    pub pointer_id: PointerId,
    /// Timestamp of the scroll event.
    pub timestamp: Instant,
}

/// Stateful scroll event recognizer and aggregator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRecognizer {
    settings: ScrollSettings,
}

impl ScrollRecognizer {
    /// Creates a scroll recognizer with custom settings.
    pub fn new(settings: ScrollSettings) -> Self {
        Self { settings }
    }

    /// Aggregates scroll events and allows each one to be handled before
    /// optional consumption.
    pub fn for_each(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        mut handle_scroll: impl FnMut(ScrollChangeContext, &mut ScrollEventContent),
    ) -> ScrollResult {
        if pass != PointerEventPass::Main {
            return ScrollResult::default();
        }

        let mut result = ScrollResult::default();
        for change in pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            let CursorEventContent::Scroll(ref mut scroll) = change.content else {
                continue;
            };
            if result.source.is_none() {
                result.source = Some(scroll.source);
            }
            if result.unit.is_none() {
                result.unit = Some(scroll.unit);
            }
            result.event_count += 1;
            result.delta_x += scroll.delta_x;
            result.delta_y += scroll.delta_y;

            handle_scroll(
                ScrollChangeContext {
                    pointer_id: change.pointer_id,
                    timestamp: change.timestamp,
                },
                scroll,
            );

            if self.settings.consume
                || (scroll.delta_x.abs() <= f32::EPSILON && scroll.delta_y.abs() <= f32::EPSILON)
            {
                change.consume();
            }
        }
        result
    }

    /// Aggregates scroll events and optionally consumes them.
    pub fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
    ) -> ScrollResult {
        if pass != PointerEventPass::Main {
            return ScrollResult::default();
        }

        let mut result = ScrollResult::default();
        for change in pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            let CursorEventContent::Scroll(ref scroll) = change.content else {
                continue;
            };
            if result.source.is_none() {
                result.source = Some(scroll.source);
            }
            if result.unit.is_none() {
                result.unit = Some(scroll.unit);
            }
            result.event_count += 1;
            result.delta_x += scroll.delta_x;
            result.delta_y += scroll.delta_y;
            if self.settings.consume {
                change.consume();
            }
        }
        result
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn cancelled_touch_clears_tap_drag_and_long_press_without_completion() {
        let mut tap = TapRecognizer::default();
        let mut drag = DragRecognizer::default();
        let mut long_press = LongPressRecognizer::default();
        let now = Instant::now();
        for content in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Cancelled(PressKeyEventType::Left),
        ] {
            let change = PointerChange::new(now, 1, content, GestureState::TapCandidate);
            let tapped = tap.update(PointerEventPass::Main, &mut [change.clone()], true);
            let dragged = drag.update(PointerEventPass::Main, &mut [change.clone()], true);
            let pressed = long_press.update(PointerEventPass::Main, &mut [change], true);
            assert!(!tapped.tapped);
            assert!(tapped.release_timestamp.is_none());
            assert!(!dragged.ended);
            assert!(!pressed.triggered);
        }
        assert!(tap.active_pointer.is_none());
        assert!(drag.active_pointer.is_none());
        assert!(long_press.active_pointer.is_none());
    }
}
impl Default for ScrollRecognizer {
    fn default() -> Self {
        Self::new(ScrollSettings::default())
    }
}

#[cfg(test)]
mod slop_tests {
    use super::*;

    const POINTER: PointerId = 7;

    fn change(content: CursorEventContent, at: Instant) -> PointerChange {
        PointerChange::new(at, POINTER, content, GestureState::TapCandidate)
    }

    fn moved(x: i32, y: i32) -> CursorEventContent {
        CursorEventContent::Moved(PxPosition::new(Px::new(x), Px::new(y)))
    }

    fn press(tap: &mut TapRecognizer, at: Instant) {
        let mut changes = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            at,
        )];
        tap.update(PointerEventPass::Main, &mut changes, true);
    }

    #[test]
    fn exclusive_taps_leave_claimed_presses_visible_to_scroll_observers() {
        let mut child = TapRecognizer::default();
        let mut parent = TapRecognizer::default();
        let mut observer = TapRecognizer::default();
        let now = Instant::now();
        for _ in 0..2 {
            let mut down = [change(
                CursorEventContent::Pressed(PressKeyEventType::Left),
                now,
            )];
            assert!(
                !child
                    .update_exclusive(PointerEventPass::Initial, &mut down, true)
                    .pressed
            );
            assert!(
                child
                    .update_exclusive(PointerEventPass::Main, &mut down, true)
                    .pressed
            );
            assert!(
                !parent
                    .update_exclusive(PointerEventPass::Main, &mut down, true)
                    .pressed
            );
            assert!(!down[0].is_consumed());
            assert!(
                observer
                    .update(PointerEventPass::Main, &mut down, true)
                    .pressed
            );
            let mut up = [change(
                CursorEventContent::Released(PressKeyEventType::Left),
                now,
            )];
            assert!(
                child
                    .update_exclusive(PointerEventPass::Main, &mut up, true)
                    .tapped
            );
            assert!(
                !parent
                    .update_exclusive(PointerEventPass::Main, &mut up, true)
                    .tapped
            );
            assert!(
                observer
                    .update(PointerEventPass::Main, &mut up, true)
                    .released
            );
        }
    }

    #[test]
    fn first_move_sample_establishes_the_slop_origin() {
        // Positionless synthetic presses retain the movement-origin fallback.
        let mut tap = TapRecognizer::default();
        let now = Instant::now();
        press(&mut tap, now);

        let mut first = [change(moved(1000, 1000), now)];
        tap.update(PointerEventPass::Main, &mut first, true);
        let mut same = [change(moved(1000, 1000), now)];
        tap.update(PointerEventPass::Main, &mut same, true);

        assert!(!tap.canceled);
    }

    #[test]
    fn recorded_press_anchors_the_first_drag_and_cancels_a_travelled_tap() {
        let now = Instant::now();
        let press = change(CursorEventContent::Pressed(PressKeyEventType::Left), now)
            .with_position(PxPosition::from_f32_xy(100.0, 200.0));
        let mut drag = DragRecognizer::default();
        let mut tap = TapRecognizer::default();
        drag.update(PointerEventPass::Main, &mut [press.clone()], true);
        tap.update(PointerEventPass::Main, &mut [press], true);
        let movement = change(moved(100, 180), now + Duration::from_millis(16));
        let result = drag.update(PointerEventPass::Main, &mut [movement.clone()], true);
        assert!(result.started && result.updated);
        assert_eq!(result.delta_y, Px::new(-20));
        tap.update(PointerEventPass::Main, &mut [movement], true);
        let result = tap.update(
            PointerEventPass::Main,
            &mut [change(
                CursorEventContent::Released(PressKeyEventType::Left),
                now + Duration::from_millis(32),
            )],
            true,
        );
        assert!(!result.tapped);
    }

    #[test]
    fn recorded_press_cancels_long_press_on_the_first_large_movement() {
        let now = Instant::now();
        let mut recognizer = LongPressRecognizer::default();
        let press = change(CursorEventContent::Pressed(PressKeyEventType::Left), now)
            .with_position(PxPosition::from_f32_xy(100.0, 200.0));
        recognizer.update(PointerEventPass::Main, &mut [press], true);
        recognizer.update(
            PointerEventPass::Main,
            &mut [change(moved(100, 180), now + Duration::from_millis(16))],
            true,
        );
        let result = recognizer.update(
            PointerEventPass::Main,
            &mut [change(moved(100, 180), now + Duration::from_millis(800))],
            true,
        );
        assert!(!result.triggered);
    }

    #[test]
    fn tap_cancels_once_travel_exceeds_slop() {
        let mut tap = TapRecognizer::new(TapSettings {
            slop_px: 8.0,
            ..Default::default()
        });
        let now = Instant::now();
        press(&mut tap, now);

        let mut origin = [change(moved(0, 0), now)];
        tap.update(PointerEventPass::Main, &mut origin, true);
        let mut within = [change(moved(4, 4), now)];
        tap.update(PointerEventPass::Main, &mut within, true);
        assert!(!tap.canceled);

        let mut beyond = [change(moved(50, 0), now)];
        tap.update(PointerEventPass::Main, &mut beyond, true);
        assert!(tap.canceled);

        let mut release = [change(
            CursorEventContent::Released(PressKeyEventType::Left),
            now,
        )];
        let result = tap.update(PointerEventPass::Main, &mut release, true);
        assert!(result.released);
        assert!(!result.tapped);
    }

    #[test]
    fn tap_completes_when_release_stays_within_slop_and_bounds() {
        let mut tap = TapRecognizer::default();
        let now = Instant::now();
        press(&mut tap, now);

        let mut origin = [change(moved(0, 0), now)];
        tap.update(PointerEventPass::Main, &mut origin, true);
        let mut release = [change(
            CursorEventContent::Released(PressKeyEventType::Left),
            now,
        )];
        let result = tap.update(PointerEventPass::Main, &mut release, true);

        assert!(result.tapped);
    }

    #[test]
    fn press_outside_bounds_does_not_begin_a_gesture() {
        let mut tap = TapRecognizer::default();
        let now = Instant::now();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            now,
        )];
        let result = tap.update(PointerEventPass::Main, &mut press, false);

        assert!(!result.pressed);
        assert!(tap.active_pointer.is_none());
    }

    #[test]
    fn drag_starts_after_slop_and_reports_sample_deltas() {
        let mut drag = DragRecognizer::new(DragSettings {
            slop_px: 8.0,
            ..Default::default()
        });
        let now = Instant::now();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            now,
        )];
        drag.update(PointerEventPass::Main, &mut press, true);

        let mut origin = [change(moved(0, 0), now)];
        let first = drag.update(PointerEventPass::Main, &mut origin, true);
        assert!(!first.started);

        let mut within = [change(moved(5, 0), now)];
        assert!(
            !drag
                .update(PointerEventPass::Main, &mut within, true)
                .started
        );

        let mut beyond = [change(moved(20, 0), now)];
        let started = drag.update(PointerEventPass::Main, &mut beyond, true);
        assert!(started.started);
        assert_eq!(started.delta_x, Px::new(15));
        assert_eq!(started.delta_y, Px::ZERO);
    }

    #[test]
    fn local_integer_drag_delta_retains_fractional_movement_in_window_units() {
        let mut drag = DragRecognizer::new(DragSettings {
            slop_px: 0.0,
            ..Default::default()
        });
        let now = Instant::now();
        let mut press = [
            change(CursorEventContent::Pressed(PressKeyEventType::Left), now)
                .with_position(PxPosition::ZERO),
        ];
        drag.update(PointerEventPass::Main, &mut press, true);
        let mut deltas = Vec::new();
        for x in 1..=5 {
            let mut movement = [change(moved(x, 0), now)];
            let result = drag.update(PointerEventPass::Main, &mut movement, true);
            assert!(result.updated);
            let transform = tessera_ui::LocalToWindow::new(
                0.625,
                PxPosition::from_f32_xy(x as f32 * 10.25, 20.5),
            )
            .unwrap();
            deltas.push(
                drag.local_integer_delta(result.delta_x, result.delta_y, transform)
                    .0
                    .0,
            );
        }
        assert_eq!(deltas, [2, 1, 2, 1, 2]);
        assert_eq!(
            deltas.iter().sum::<i32>(),
            8,
            "Five window pixels must preserve exactly eight local pixels"
        );
    }

    #[test]
    fn horizontal_drag_ignores_vertical_travel() {
        let mut drag = DragRecognizer::new(DragSettings {
            slop_px: 8.0,
            axis: Some(DragAxis::Horizontal),
            ..Default::default()
        });
        let now = Instant::now();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            now,
        )];
        drag.update(PointerEventPass::Main, &mut press, true);
        let mut origin = [change(moved(0, 0), now)];
        drag.update(PointerEventPass::Main, &mut origin, true);

        let mut vertical = [change(moved(0, 40), now)];
        let result = drag.update(PointerEventPass::Main, &mut vertical, true);

        assert!(!result.started);
        assert!(!drag.is_dragging());
    }

    #[test]
    fn long_press_triggers_after_the_threshold() {
        let mut long_press = LongPressRecognizer::default();
        let start = Instant::now();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            start,
        )];
        long_press.update(PointerEventPass::Main, &mut press, true);
        let mut origin = [change(moved(0, 0), start)];
        long_press.update(PointerEventPass::Main, &mut origin, true);

        let mut hold = [change(moved(0, 0), start + Duration::from_millis(600))];
        let result = long_press.update(PointerEventPass::Main, &mut hold, true);

        assert!(result.triggered);
    }

    #[test]
    fn frame_long_press_deadline_is_one_shot_and_respects_cancellation_and_pointer_identity() {
        let start = Instant::now();
        let mut recognizer = LongPressRecognizer::default();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            start,
        )];
        recognizer.update_at_frame_nanos(PointerEventPass::Main, &mut press, true, 10_000_000);
        assert!(!recognizer.advance_frame_nanos(509_999_999));
        let mut other = [PointerChange::new(
            start,
            POINTER + 1,
            moved(100, 100),
            GestureState::TapCandidate,
        )];
        recognizer.update_at_frame_nanos(PointerEventPass::Main, &mut other, false, 400_000_000);
        assert!(recognizer.advance_frame_nanos(510_000_000));
        assert!(recognizer.has_triggered());
        assert!(!recognizer.needs_frame());
        assert!(!recognizer.advance_frame_nanos(900_000_000));
        let mut release = [change(
            CursorEventContent::Released(PressKeyEventType::Left),
            start,
        )];
        let result = recognizer.update_at_frame_nanos(
            PointerEventPass::Main,
            &mut release,
            true,
            900_000_000,
        );
        assert!(result.released);
        assert!(!result.triggered);
        assert!(!recognizer.is_active());
        for cancelled in [true, false] {
            let mut press = [change(
                CursorEventContent::Pressed(PressKeyEventType::Left),
                start,
            )];
            recognizer.update_at_frame_nanos(
                PointerEventPass::Main,
                &mut press,
                true,
                1_000_000_000,
            );
            let mut end = [change(
                if cancelled {
                    CursorEventContent::Cancelled(PressKeyEventType::Left)
                } else {
                    moved(0, 0)
                },
                start,
            )];
            if !cancelled {
                end[0].gesture_state = GestureState::Dragged;
            }
            recognizer.update_at_frame_nanos(
                if cancelled {
                    PointerEventPass::Main
                } else {
                    PointerEventPass::Final
                },
                &mut end,
                true,
                1_100_000_000,
            );
            assert!(!recognizer.needs_frame());
            assert!(!recognizer.advance_frame_nanos(2_000_000_000));
        }
    }

    #[test]
    fn frame_long_press_observes_owned_consumed_release_and_event_time_backlogs() {
        let start = Instant::now();
        let mut tap = TapRecognizer::new(TapSettings {
            consume_on_tap: true,
            ..Default::default()
        });
        let mut long = LongPressRecognizer::default();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            start,
        )];
        let result = tap.update_exclusive(PointerEventPass::Main, &mut press, true);
        long.update_after_tap(PointerEventPass::Main, &mut press, true, 0, result);
        let mut release = [change(
            CursorEventContent::Released(PressKeyEventType::Left),
            start + Duration::from_millis(600),
        )];
        let result = tap.update_exclusive(PointerEventPass::Main, &mut release, true);
        assert!(result.tapped);
        assert!(release[0].is_consumed());
        let long_result =
            long.update_after_tap(PointerEventPass::Main, &mut release, true, 0, result);
        assert!(long_result.triggered);
        assert!(long_result.released);
        assert!(!long.is_active());
        let mut child = TapRecognizer::default();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            start,
        )];
        child.update_exclusive(PointerEventPass::Main, &mut press, true);
        let result = tap.update_exclusive(PointerEventPass::Main, &mut press, true);
        assert!(!result.pressed);
        long.update_after_tap(PointerEventPass::Main, &mut press, true, 0, result);
        assert!(!long.needs_frame());
    }

    #[test]
    fn long_press_is_canceled_by_travel_beyond_slop() {
        let mut long_press = LongPressRecognizer::default();
        let start = Instant::now();
        let mut press = [change(
            CursorEventContent::Pressed(PressKeyEventType::Left),
            start,
        )];
        long_press.update(PointerEventPass::Main, &mut press, true);
        let mut origin = [change(moved(0, 0), start)];
        long_press.update(PointerEventPass::Main, &mut origin, true);

        let mut beyond = [change(moved(40, 0), start)];
        long_press.update(PointerEventPass::Main, &mut beyond, true);
        assert!(long_press.canceled);

        let mut hold = [change(moved(40, 0), start + Duration::from_millis(600))];
        let result = long_press.update(PointerEventPass::Main, &mut hold, true);
        assert!(!result.triggered);
    }
}
