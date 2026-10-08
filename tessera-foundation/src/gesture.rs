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
/// A press event carries no position, so the origin of a gesture is the
/// **first movement sample** observed after the press. `PressTrack` owns that
/// origin together with the most recent sample and derives every slop and
/// delta value the recognizers need from event positions alone, so tap,
/// long-press, and drag no longer each carry their own copy of this logic.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct PressTrack {
    /// Slop origin: the first movement sample seen since the press.
    start: Option<PxPosition>,
    /// Most recent movement sample, used as the delta baseline.
    last: Option<PxPosition>,
}

impl PressTrack {
    /// Clears the track so the next movement sample becomes the new origin.
    ///
    /// Called both when a fresh press starts and when a recognizer resets.
    fn on_press(&mut self) {
        self.start = None;
        self.last = None;
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
                    self.active_pointer = Some(change.pointer_id);
                    self.track.on_press();
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

    fn reset(&mut self) {
        self.active_pointer = None;
        self.track.on_press();
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
    settings: DragSettings,
    active_pointer: Option<PointerId>,
    track: PressTrack,
    dragging: bool,
}

impl DragRecognizer {
    /// Creates a drag recognizer with custom settings.
    pub fn new(settings: DragSettings) -> Self {
        Self {
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
                    self.active_pointer = Some(change.pointer_id);
                    self.track.on_press();
                    self.dragging = false;
                }
                CursorEventContent::Moved(position) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    let observation = self.track.observe(position, self.settings.slop_px);
                    if observation.started_now {
                        // The press position is not carried by the event, so
                        // the first sample becomes both
                        // the slop origin and the delta
                        // baseline.
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

    fn reset(&mut self) {
        self.active_pointer = None;
        self.track.on_press();
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
        if pass != PointerEventPass::Main {
            return LongPressResult::default();
        }

        let mut result = LongPressResult::default();
        for change in pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) if within_bounds => {
                    self.active_pointer = Some(change.pointer_id);
                    self.track.on_press();
                    self.press_time = Some(change.timestamp);
                    self.canceled = false;
                    self.triggered = false;
                }
                CursorEventContent::Moved(position) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    let observation = self.track.observe(position, self.settings.slop_px);
                    if observation.exceeded_slop {
                        self.canceled = true;
                    }
                    self.try_trigger(change.timestamp, within_bounds, change, &mut result);
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
                    self.try_trigger(change.timestamp, within_bounds, change, &mut result);
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
        change: &mut PointerChange,
        result: &mut LongPressResult,
    ) {
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
        self.track.on_press();
        self.press_time = None;
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
    fn first_move_sample_establishes_the_slop_origin() {
        // The press event carries no position, so a far-away first sample only
        // fixes the origin and must not count as travel.
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
