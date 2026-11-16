//! Compose-style asynchronous gesture detectors.
//!
//! ```
//! use tessera_foundation::{
//!     modifier::ModifierExt as _,
//!     pointer_gestures::{TapGesture, TapGestureConfig, detect_tap_gestures},
//! };
//! use tessera_ui::{
//!     Dp, Modifier, PxPosition, State, tessera,
//!     layout::{Stack, layout}, testing::UiTest,
//! };
//!
//! #[tessera]
//! fn tap_target(count: State<usize>) {
//!     layout(Stack).modifier(
//!         Modifier::new().size(Dp(80.0), Dp(80.0))
//!             .pointer_input_once(move |mut scope| async move {
//!                 detect_tap_gestures(&mut scope, TapGestureConfig::default(), |gesture| {
//!                     if matches!(gesture, TapGesture::Tapped(_, _)) {
//!                         count.with_mut(|count| *count += 1);
//!                     }
//!                 }).await;
//!             }),
//!     );
//! }
//!
//! let test = UiTest::new();
//! let count = test.state(0);
//! let mut test = test.content(|| tap_target(count));
//! test.frame();
//! test.click_at(PxPosition::from_f32_xy(10.0, 10.0));
//! assert_eq!(count.get(), 1);
//! ```

#[cfg(test)]
mod tests;

use std::time::Duration;

use tessera_ui::{
    CursorEventContent, DownRequirement, GestureEnd, LongPressResult, PointerChange, PointerDown,
    PointerEvent, PointerEventPass, PointerInputScope, PressKeyEventType, PxPosition,
    ScrollEventContent,
};

/// Configuration for [`detect_tap_gestures`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapGestureConfig {
    /// Button that starts the gesture.
    pub button: PressKeyEventType,
    /// Maximum movement before the tap is cancelled.
    pub slop_px: f32,
    /// Whether to consume the press as soon as it is accepted.
    pub consume_down: bool,
    /// Whether to consume a completed release.
    pub consume_up: bool,
    /// Optional hold duration that replaces a completed tap with a long press.
    pub long_press: Option<Duration>,
}

impl Default for TapGestureConfig {
    fn default() -> Self {
        Self {
            button: PressKeyEventType::Left,
            slop_px: 8.0,
            consume_down: false,
            consume_up: false,
            long_press: None,
        }
    }
}

/// A transition reported by [`detect_tap_gestures`].
pub enum TapGesture {
    /// An exclusive press was accepted.
    Pressed(PointerDown),
    /// The press ended normally, including after a long press.
    Released(PointerDown, PointerChange),
    /// A press and release completed without slop or cancellation.
    Tapped(PointerDown, PointerChange),
    /// The configured hold duration elapsed.
    LongPressed(PointerDown),
    /// Input was cancelled, claimed elsewhere, or the detector was dropped.
    Cancelled,
}

struct TapObserver<F: FnMut(TapGesture)> {
    callback: F,
    pressed: bool,
}

impl<F: FnMut(TapGesture)> Drop for TapObserver<F> {
    fn drop(&mut self) {
        if self.pressed {
            (self.callback)(TapGesture::Cancelled);
        }
    }
}

/// Detects exclusive taps and optional long presses, including cancellation.
pub async fn detect_tap_gestures(
    scope: &mut PointerInputScope,
    config: TapGestureConfig,
    on_gesture: impl FnMut(TapGesture),
) {
    let mut observer = TapObserver {
        callback: on_gesture,
        pressed: false,
    };
    loop {
        let Some(mut down) = scope
            .await_first_down(
                PointerEventPass::Main,
                Some(config.button),
                DownRequirement::Unconsumed,
            )
            .await
        else {
            return;
        };
        if !down.change.claim_tap_press() {
            continue;
        }
        if config.consume_down {
            down.change.consume();
        }
        observer.pressed = true;
        (observer.callback)(TapGesture::Pressed(down.clone()));
        let result = if let Some(timeout) = config.long_press {
            scope
                .await_long_press_or_cancellation(
                    PointerEventPass::Main,
                    &down,
                    timeout,
                    config.slop_px,
                )
                .await
        } else {
            wait_for_tap(scope, &down, config).await
        };
        let (up, tapped) = match result {
            LongPressResult::Released(up) => (Some(up), true),
            LongPressResult::Cancelled => (None, false),
            LongPressResult::Triggered { .. } => {
                (observer.callback)(TapGesture::LongPressed(down.clone()));
                let end = scope
                    .consume_until_up(PointerEventPass::Main, down.change.pointer_id)
                    .await;
                (
                    match end {
                        GestureEnd::Released(up) => Some(up),
                        GestureEnd::Cancelled => None,
                    },
                    false,
                )
            }
        };
        observer.pressed = false;
        if let Some(mut up) = up {
            if config.consume_up {
                up.consume();
            }
            (observer.callback)(TapGesture::Released(down.clone(), up.clone()));
            if tapped {
                (observer.callback)(TapGesture::Tapped(down, up));
            }
        } else {
            (observer.callback)(TapGesture::Cancelled);
        }
    }
}

async fn wait_for_tap(
    scope: &mut PointerInputScope,
    down: &PointerDown,
    config: TapGestureConfig,
) -> LongPressResult {
    loop {
        let event = scope.await_pointer_event(PointerEventPass::Main).await;
        if event.is_cancelled() {
            return LongPressResult::Cancelled;
        }
        let within = event.is_pointer_inside();
        for change in event.changes {
            if change.pointer_id != down.change.pointer_id {
                continue;
            }
            if change.is_consumed()
                || !within
                || down
                    .position
                    .zip(change.position())
                    .is_some_and(|(start, point)| start.distance_to(point) > config.slop_px)
            {
                return LongPressResult::Cancelled;
            }
            match change.content {
                CursorEventContent::Released(button) if button == config.button => {
                    return if change.gesture_state == tessera_ui::GestureState::TapCandidate {
                        LongPressResult::Released(change)
                    } else {
                        LongPressResult::Cancelled
                    };
                }
                CursorEventContent::Cancelled(_) | CursorEventContent::Scroll(_) => {
                    return LongPressResult::Cancelled;
                }
                _ => {}
            }
        }
        let final_event = scope.await_pointer_event(PointerEventPass::Final).await;
        if final_event.is_cancelled()
            || final_event
                .changes
                .iter()
                .any(|change| change.pointer_id == down.change.pointer_id && change.is_consumed())
        {
            return LongPressResult::Cancelled;
        }
    }
}

/// Configuration for [`detect_long_press_gestures`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongPressGestureConfig {
    /// Pass used to observe the press and its subsequent changes.
    pub pass: PointerEventPass,
    /// Whether a press already consumed by a child can be observed.
    pub down_requirement: DownRequirement,
    /// Button that starts the gesture.
    pub button: PressKeyEventType,
    /// Time the pointer must remain down.
    pub threshold: Duration,
    /// Movement threshold that cancels the gesture.
    pub slop_px: f32,
    /// Whether to consume subsequent changes through release after triggering.
    pub consume_trigger: bool,
}

impl Default for LongPressGestureConfig {
    fn default() -> Self {
        Self {
            pass: PointerEventPass::Main,
            down_requirement: DownRequirement::Unconsumed,
            button: PressKeyEventType::Left,
            threshold: Duration::from_millis(500),
            slop_px: 8.0,
            consume_trigger: false,
        }
    }
}

/// Waits for long presses until the pointer-input task is cancelled.
pub async fn detect_long_press_gestures(
    scope: &mut PointerInputScope,
    config: LongPressGestureConfig,
    mut on_long_press: impl FnMut(&PointerDown),
) {
    loop {
        let Some(down) = scope
            .await_first_down(config.pass, Some(config.button), config.down_requirement)
            .await
        else {
            return;
        };
        match scope
            .await_long_press_or_cancellation(config.pass, &down, config.threshold, config.slop_px)
            .await
        {
            LongPressResult::Triggered { .. } => on_long_press(&down),
            LongPressResult::Released(_) | LongPressResult::Cancelled => continue,
        }
        match if config.consume_trigger {
            scope
                .consume_until_up(config.pass, down.change.pointer_id)
                .await
        } else {
            scope
                .wait_for_up_or_cancellation(config.pass, down.change.pointer_id)
                .await
        } {
            GestureEnd::Released(mut up) if config.consume_trigger => up.consume(),
            GestureEnd::Released(_) | GestureEnd::Cancelled => {}
        }
    }
}

/// Configuration for [`detect_drag_gestures`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragGestureConfig {
    /// Minimum movement before dragging starts.
    pub slop_px: f32,
    /// Optional axis lock.
    pub axis: Option<DragAxis>,
    /// Whether movement changes are consumed after the drag starts.
    pub consume: bool,
}

/// Axis lock for drag detection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DragAxis {
    /// Keep horizontal movement and discard vertical movement.
    Horizontal,
    /// Keep vertical movement and discard horizontal movement.
    Vertical,
}

impl Default for DragGestureConfig {
    fn default() -> Self {
        Self {
            slop_px: 8.0,
            axis: None,
            consume: true,
        }
    }
}

struct GestureCancellation<'a, F: FnMut()> {
    callback: &'a mut F,
    active: bool,
}

impl<F: FnMut()> Drop for GestureCancellation<'_, F> {
    fn drop(&mut self) {
        if self.active {
            (self.callback)();
        }
    }
}

/// Detects drags, reporting local deltas and the sample that produced them.
///
/// Presses remain visible to child controls. Movement is claimed in Main after
/// crossing slop from the press position. `on_cancel` also runs when an active
/// detector is dropped; captured state must still be alive before accessing it.
pub async fn detect_drag_gestures(
    scope: &mut PointerInputScope,
    config: DragGestureConfig,
    mut on_start: impl FnMut(&PointerDown),
    mut on_delta: impl FnMut(&PointerEvent, &PointerChange, PxPosition),
    mut on_stop: impl FnMut(&PointerEvent, &PointerChange),
    mut on_cancel: impl FnMut(),
) {
    loop {
        let Some(mut down) = scope
            .await_first_down(
                if config.slop_px <= 0.0 {
                    PointerEventPass::Main
                } else {
                    PointerEventPass::Initial
                },
                Some(PressKeyEventType::Left),
                if config.slop_px <= 0.0 {
                    DownRequirement::Unconsumed
                } else {
                    DownRequirement::Any
                },
            )
            .await
        else {
            return;
        };
        let mut start = down.position;
        let mut previous = start;
        let mut cancellation = GestureCancellation {
            callback: &mut on_cancel,
            active: false,
        };
        if config.slop_px <= 0.0 {
            on_start(&down);
            cancellation.active = true;
            if config.consume {
                down.change.consume();
            }
        }
        'gesture: loop {
            let event = scope.await_pointer_event(PointerEventPass::Main).await;
            if event.is_cancelled() {
                return;
            }
            for mut change in event.changes.iter().cloned() {
                if change.pointer_id != down.change.pointer_id {
                    continue;
                }
                match change.content {
                    CursorEventContent::Moved(position) => {
                        if change.is_consumed() {
                            break 'gesture;
                        }
                        let travel = position - *start.get_or_insert(position);
                        let mut delta = position - previous.replace(position).unwrap_or(position);
                        if !cancellation.active {
                            let distance = match config.axis {
                                Some(DragAxis::Horizontal) => travel.x.abs(),
                                Some(DragAxis::Vertical) => travel.y.abs(),
                                None => travel.x.hypot(travel.y),
                            };
                            if distance < config.slop_px {
                                continue;
                            }
                            on_start(&down);
                            cancellation.active = true;
                        }
                        match config.axis {
                            Some(DragAxis::Horizontal) => delta.y = 0.0,
                            Some(DragAxis::Vertical) => delta.x = 0.0,
                            None => {}
                        }
                        on_delta(&event, &change, event.local_to_window.inverse_vector(delta));
                        if config.consume {
                            change.consume();
                        }
                    }
                    CursorEventContent::Released(PressKeyEventType::Left) => {
                        if cancellation.active && !change.is_consumed() {
                            cancellation.active = false;
                            on_stop(&event, &change);
                            if config.consume {
                                change.consume();
                            }
                        }
                        break 'gesture;
                    }
                    CursorEventContent::Cancelled(PressKeyEventType::Left) => break 'gesture,
                    _ => {}
                }
            }
        }
    }
}

/// Configuration for [`detect_scroll_gestures`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollGestureConfig {
    /// Whether scroll changes are consumed after the callback.
    pub consume: bool,
}

/// Observes scroll changes until the pointer-input task is cancelled.
pub async fn detect_scroll_gestures(
    scope: &mut PointerInputScope,
    config: ScrollGestureConfig,
    mut on_scroll: impl FnMut(&PointerEvent, &PointerChange, &mut ScrollEventContent),
) {
    loop {
        let event = scope.await_pointer_event(PointerEventPass::Main).await;
        if event.is_cancelled() {
            return;
        }
        for mut change in event.changes.iter().cloned() {
            if change.is_consumed() {
                continue;
            }
            if let CursorEventContent::Scroll(mut scroll) = change.content.clone() {
                on_scroll(&event, &change, &mut scroll);
                let exhausted =
                    scroll.delta_x.abs() < f32::EPSILON && scroll.delta_y.abs() < f32::EPSILON;
                event.set_scroll_remainder(change.pointer_id, scroll);
                if config.consume || exhausted {
                    change.consume();
                }
            }
        }
    }
}
