//! Gesture recognizers for pointer-driven interactions.
//!
//! ## Usage
//!
//! Use these recognizers to derive tap, drag, long-press, and scroll behavior.

use std::time::Duration;

use tessera_ui::{
    CursorEventContent, GestureState, PointerChange, PointerEventPass, PointerId,
    PressKeyEventType, Px, PxPosition, ScrollEventContent, ScrollEventSource, time::Instant,
};

const DEFAULT_SLOP_PX: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TapSettings {
    pub button: PressKeyEventType,
    pub slop_px: f32,
    pub consume_on_press: bool,
    pub consume_on_release: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct TapResult {
    pub pressed: bool,
    pub released: bool,
    pub tapped: bool,
    pub press_timestamp: Option<Instant>,
    pub release_timestamp: Option<Instant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TapRecognizer {
    settings: TapSettings,
    active_pointer: Option<PointerId>,
    press_position: Option<PxPosition>,
    canceled: bool,
}

impl TapRecognizer {
    pub(crate) fn new(settings: TapSettings) -> Self {
        Self {
            settings,
            active_pointer: None,
            press_position: None,
            canceled: false,
        }
    }

    pub(crate) fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        cursor_position: Option<PxPosition>,
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
                CursorEventContent::Pressed(button) if button == self.settings.button => {
                    if within_bounds {
                        self.active_pointer = Some(change.pointer_id);
                        self.press_position = cursor_position;
                        self.canceled = false;
                        result.pressed = true;
                        result.press_timestamp = Some(change.timestamp);
                        if self.settings.consume_on_press {
                            change.consume();
                        }
                    }
                }
                CursorEventContent::Moved(_) => {
                    if Some(change.pointer_id) == self.active_pointer
                        && let Some(start) = self.press_position
                        && let Some(position) = cursor_position
                        && start.distance_to(position) > self.settings.slop_px
                    {
                        self.canceled = true;
                    }
                }
                CursorEventContent::Scroll(_) => {
                    if Some(change.pointer_id) == self.active_pointer {
                        self.canceled = true;
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
        self.press_position = None;
        self.canceled = false;
    }
}

impl Default for TapRecognizer {
    fn default() -> Self {
        Self::new(TapSettings::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DragSettings {
    pub slop_px: f32,
    pub consume_when_dragging: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DragAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct DragResult {
    pub started: bool,
    pub updated: bool,
    pub ended: bool,
    pub delta_x: Px,
    pub delta_y: Px,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DragRecognizer {
    settings: DragSettings,
    active_pointer: Option<PointerId>,
    start_position: Option<PxPosition>,
    last_position: Option<PxPosition>,
    dragging: bool,
}

impl DragRecognizer {
    pub(crate) fn new(settings: DragSettings) -> Self {
        Self {
            settings,
            active_pointer: None,
            start_position: None,
            last_position: None,
            dragging: false,
        }
    }

    pub(crate) fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        cursor_position: Option<PxPosition>,
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
                CursorEventContent::Pressed(PressKeyEventType::Left) => {
                    if within_bounds {
                        self.active_pointer = Some(change.pointer_id);
                        self.start_position = cursor_position;
                        self.last_position = cursor_position;
                        self.dragging = false;
                    }
                }
                CursorEventContent::Moved(_) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    let Some(position) = cursor_position else {
                        continue;
                    };
                    if !self.dragging
                        && let Some(start) = self.start_position
                    {
                        let delta_x = (position.x - start.x).to_f32();
                        let delta_y = (position.y - start.y).to_f32();
                        let should_start = match self.settings.axis {
                            Some(DragAxis::Horizontal) => {
                                delta_x.abs() > self.settings.slop_px
                                    && delta_x.abs() >= delta_y.abs()
                            }
                            Some(DragAxis::Vertical) => {
                                delta_y.abs() > self.settings.slop_px
                                    && delta_y.abs() >= delta_x.abs()
                            }
                            None => start.distance_to(position) > self.settings.slop_px,
                        };
                        if should_start {
                            self.dragging = true;
                            result.started = true;
                        }
                    }
                    if self.dragging {
                        if let Some(last) = self.last_position {
                            let raw_dx = position.x - last.x;
                            let raw_dy = position.y - last.y;
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
                        }
                        self.last_position = Some(position);
                        if self.settings.consume_when_dragging {
                            change.consume();
                        }
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

    fn reset(&mut self) {
        self.active_pointer = None;
        self.start_position = None;
        self.last_position = None;
        self.dragging = false;
    }
}

impl Default for DragRecognizer {
    fn default() -> Self {
        Self::new(DragSettings::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LongPressSettings {
    pub threshold: Duration,
    pub slop_px: f32,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct LongPressResult {
    pub triggered: bool,
    pub released: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LongPressRecognizer {
    settings: LongPressSettings,
    active_pointer: Option<PointerId>,
    press_position: Option<PxPosition>,
    press_time: Option<Instant>,
    canceled: bool,
    triggered: bool,
}

impl LongPressRecognizer {
    pub(crate) fn new(settings: LongPressSettings) -> Self {
        Self {
            settings,
            active_pointer: None,
            press_position: None,
            press_time: None,
            canceled: false,
            triggered: false,
        }
    }

    pub(crate) fn update(
        &mut self,
        pass: PointerEventPass,
        pointer_changes: &mut [PointerChange],
        cursor_position: Option<PxPosition>,
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
                CursorEventContent::Pressed(PressKeyEventType::Left) => {
                    if within_bounds {
                        self.active_pointer = Some(change.pointer_id);
                        self.press_position = cursor_position;
                        self.press_time = Some(change.timestamp);
                        self.canceled = false;
                        self.triggered = false;
                    }
                }
                CursorEventContent::Moved(_) => {
                    if Some(change.pointer_id) != self.active_pointer {
                        continue;
                    }
                    let Some(position) = cursor_position else {
                        continue;
                    };
                    if let Some(start) = self.press_position
                        && start.distance_to(position) > self.settings.slop_px
                    {
                        self.canceled = true;
                    }
                    self.try_trigger(change.timestamp, within_bounds, change, &mut result);
                }
                CursorEventContent::Scroll(_) => {
                    if Some(change.pointer_id) == self.active_pointer {
                        self.canceled = true;
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
        self.press_position = None;
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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct ScrollSettings {
    pub consume: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollResult {
    pub event_count: usize,
    pub delta_x: f32,
    pub delta_y: f32,
    pub source: Option<ScrollEventSource>,
}

impl Default for ScrollResult {
    fn default() -> Self {
        Self {
            event_count: 0,
            delta_x: 0.0,
            delta_y: 0.0,
            source: None,
        }
    }
}

impl ScrollResult {
    pub(crate) fn has_scroll(&self) -> bool {
        self.event_count > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollChangeContext {
    pub pointer_id: PointerId,
    pub timestamp: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollRecognizer {
    settings: ScrollSettings,
}

impl ScrollRecognizer {
    pub(crate) fn new(settings: ScrollSettings) -> Self {
        Self { settings }
    }

    pub(crate) fn for_each(
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

    pub(crate) fn update(
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

impl Default for ScrollRecognizer {
    fn default() -> Self {
        Self::new(ScrollSettings::default())
    }
}
