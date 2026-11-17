//! Asynchronous pointer input for modifier-owned gesture tasks.

use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures_util::future::{Either, select};

use crate::{
    CursorEventContent, Modifier, PointerChange, PointerEventPass, PointerId, PointerInput,
    PressKeyEventType,
    modifier::{BuildModifierNode, ModifierBuildContext, PointerInputModifierNode},
};

/// A pointer event delivered to an asynchronous pointer-input task.
///
/// The changes retain the live consumption transaction for the dispatch pass
/// that produced them. Calling [`PointerChange::consume`] therefore affects
/// handlers that run later in that pass.
#[derive(Clone, Debug)]
pub struct PointerEvent {
    /// Dispatch pass in which this event was observed.
    pub pass: PointerEventPass,
    /// Pointer changes observed by the modifier.
    pub changes: Vec<PointerChange>,
    /// Cursor position in the modifier's local coordinate space.
    pub cursor_position: Option<crate::PxPosition>,
    /// Absolute cursor position sampled for the dispatch.
    pub window_cursor_position: Option<crate::PxPosition>,
    /// Measured local size of the node that received the event.
    pub size: crate::PxSize,
    /// Maps local positions and vectors to window coordinates.
    pub local_to_window: crate::LocalToWindow,
    /// Keyboard modifiers sampled with this pointer event.
    pub key_modifiers: winit::keyboard::ModifiersState,
    commands: Rc<PointerDispatchCommands>,
    cancelled: bool,
}

impl PointerEvent {
    /// Returns whether the sample lies inside this modifier's local bounds.
    pub fn is_pointer_inside(&self) -> bool {
        self.cursor_position.is_some_and(|position| {
            position.x >= 0.0
                && position.y >= 0.0
                && position.x < self.size.width.to_f32()
                && position.y < self.size.height.to_f32()
        })
    }

    fn cancelled(pass: PointerEventPass) -> Self {
        Self {
            pass,
            changes: Vec::new(),
            cursor_position: None,
            window_cursor_position: None,
            size: crate::PxSize::ZERO,
            local_to_window: crate::LocalToWindow::IDENTITY,
            key_modifiers: winit::keyboard::ModifiersState::empty(),
            commands: Rc::new(PointerDispatchCommands::default()),
            cancelled: true,
        }
    }

    /// Returns whether the modifier was detached while waiting for input.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Blocks pointer changes from reaching later components in this dispatch.
    pub fn block_cursor(&self) {
        self.commands.block_cursor.set(true);
    }

    /// Starts a system window drag for the current press.
    pub fn drag_window(&self) {
        self.commands.drag_window.set(true);
    }

    /// Requests the input method to open for this pointer dispatch.
    pub fn show_ime(&self, request: crate::ImeRequest) {
        *self.commands.ime_request.borrow_mut() = Some((request, true));
    }

    /// Updates the input method state without requesting it to open.
    pub fn update_ime(&self, request: crate::ImeRequest) {
        *self.commands.ime_request.borrow_mut() = Some((request, false));
    }

    /// Publishes the scroll remainder for a pointer so ancestor handlers in
    /// this dispatch pass observe the updated delta.
    pub fn set_scroll_remainder(
        &self,
        pointer_id: PointerId,
        remainder: crate::ScrollEventContent,
    ) {
        self.commands
            .scroll_remainders
            .borrow_mut()
            .insert(pointer_id, remainder);
    }
}

#[derive(Debug, Default)]
struct PointerDispatchCommands {
    block_cursor: Cell<bool>,
    drag_window: Cell<bool>,
    scroll_remainders: RefCell<std::collections::HashMap<PointerId, crate::ScrollEventContent>>,
    ime_request: RefCell<Option<(crate::ImeRequest, bool)>>,
}

/// A press together with the local position at which it was observed.
#[derive(Clone, Debug)]
pub struct PointerDown {
    /// The underlying pointer change.
    pub change: PointerChange,
    /// Window position of the press, when available.
    pub position: Option<crate::PxPosition>,
    /// Local position of the press, when available.
    pub local_position: Option<crate::PxPosition>,
    /// Measured local size of the node that received the press.
    pub size: crate::PxSize,
    /// Coordinate transform at the press sample.
    pub local_to_window: crate::LocalToWindow,
    commands: Rc<PointerDispatchCommands>,
}

impl PointerDown {
    /// Blocks pointer changes from reaching later components in this dispatch.
    pub fn block_cursor(&self) {
        self.commands.block_cursor.set(true);
    }

    /// Starts a system window drag for the current press.
    pub fn drag_window(&self) {
        self.commands.drag_window.set(true);
    }

    /// Publishes the scroll remainder for a pointer so ancestor handlers in
    /// this dispatch pass observe the updated delta.
    pub fn set_scroll_remainder(
        &self,
        pointer_id: PointerId,
        remainder: crate::ScrollEventContent,
    ) {
        self.commands
            .scroll_remainders
            .borrow_mut()
            .insert(pointer_id, remainder);
    }

    /// Requests the input method to open for this pointer dispatch.
    pub fn show_ime(&self, request: crate::ImeRequest) {
        *self.commands.ime_request.borrow_mut() = Some((request, true));
    }

    /// Updates the input method state without requesting it to open.
    pub fn update_ime(&self, request: crate::ImeRequest) {
        *self.commands.ime_request.borrow_mut() = Some((request, false));
    }
}

/// Controls whether [`PointerInputScope::await_first_down`] accepts consumed
/// press changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownRequirement {
    /// Accept both consumed and unconsumed presses.
    Any,
    /// Accept only presses that are still unconsumed.
    Unconsumed,
}

/// Result of waiting for a pointer to be released or cancelled.
#[derive(Clone, Debug)]
pub enum GestureEnd {
    /// The requested pointer was released.
    Released(PointerChange),
    /// The gesture was cancelled before release.
    Cancelled,
}

/// Result of waiting for a long press.
#[derive(Clone, Debug)]
pub enum LongPressResult {
    /// The timeout elapsed while the pointer remained active.
    Triggered {
        /// Last known pointer position when the timeout elapsed.
        position: Option<crate::PxPosition>,
    },
    /// The pointer was released before the timeout.
    Released(PointerChange),
    /// The gesture was cancelled.
    Cancelled,
}

pub(crate) struct PointerEventQueue {
    event: RefCell<Option<PointerEvent>>,
    pressed: RefCell<Vec<(PointerId, PressKeyEventType)>>,
    waiter: RefCell<Option<Waker>>,
    pass: Cell<Option<PointerEventPass>>,
    cancelled: Cell<bool>,
}

impl PointerEventQueue {
    fn publish(&self, event: PointerEvent) {
        if self.pass.get() != Some(event.pass) {
            return;
        }
        *self.event.borrow_mut() = Some(event);
        if let Some(waker) = self.waiter.borrow_mut().take() {
            waker.wake();
        }
    }

    fn cancel(&self) {
        self.cancelled.set(true);
        if let Some(waker) = self.waiter.borrow_mut().take() {
            waker.wake();
        }
    }
}

/// A node-local asynchronous pointer event scope.
///
/// The scope is created by [`Modifier::pointer_input`] or
/// [`Modifier::pointer_input_once`]. It is local to the UI session and is
/// cancelled automatically when its modifier is detached or restarted.
pub struct PointerInputScope {
    queue: Rc<PointerEventQueue>,
}

impl PointerInputScope {
    pub(crate) fn new(queue: Rc<PointerEventQueue>) -> Self {
        Self { queue }
    }

    /// Waits for the next event delivered in `pass`.
    pub async fn await_pointer_event(&mut self, pass: PointerEventPass) -> PointerEvent {
        AwaitPointerEvent {
            queue: self.queue.clone(),
            pass,
        }
        .await
    }

    /// Waits for a press matching `button` and `requirement`.
    pub async fn await_first_down(
        &mut self,
        pass: PointerEventPass,
        button: Option<PressKeyEventType>,
        requirement: DownRequirement,
    ) -> Option<PointerDown> {
        loop {
            let event = self.await_pointer_event(pass).await;
            if event.is_cancelled() {
                return None;
            }
            if !event.is_pointer_inside() {
                continue;
            }
            if let Some(change) = event.changes.into_iter().find(|change| {
                matches!(change.content, CursorEventContent::Pressed(value)
                    if button.is_none_or(|button| button == value))
                    && (requirement == DownRequirement::Any || !change.is_consumed())
            }) {
                return Some(PointerDown {
                    position: change.position().or(event.window_cursor_position),
                    change,
                    local_position: event.cursor_position,
                    size: event.size,
                    local_to_window: event.local_to_window,
                    commands: event.commands,
                });
            }
        }
    }

    /// Waits for `pointer_id` to be released, returning cancellation for any
    /// other terminal condition.
    pub async fn wait_for_up_or_cancellation(
        &mut self,
        pass: PointerEventPass,
        pointer_id: PointerId,
    ) -> GestureEnd {
        loop {
            let event = self.await_pointer_event(pass).await;
            if event.is_cancelled() {
                return GestureEnd::Cancelled;
            }
            for change in event.changes {
                if change.pointer_id != pointer_id {
                    continue;
                }
                match change.content {
                    CursorEventContent::Released(_) => return GestureEnd::Released(change),
                    CursorEventContent::Cancelled(_) => return GestureEnd::Cancelled,
                    _ => {}
                }
            }
        }
    }

    /// Consumes subsequent changes for `pointer_id` until release or
    /// cancellation.
    pub async fn consume_until_up(
        &mut self,
        pass: PointerEventPass,
        pointer_id: PointerId,
    ) -> GestureEnd {
        loop {
            let event = self.await_pointer_event(pass).await;
            if event.is_cancelled() {
                return GestureEnd::Cancelled;
            }
            for mut change in event.changes {
                if change.pointer_id != pointer_id {
                    continue;
                }
                match change.content {
                    CursorEventContent::Released(_) => {
                        change.consume();
                        return GestureEnd::Released(change);
                    }
                    CursorEventContent::Cancelled(_) => return GestureEnd::Cancelled,
                    _ => change.consume(),
                }
            }
        }
    }

    /// Waits until the pointer moves beyond `slop`, is released, or is
    /// cancelled. The returned displacement is measured from `down`.
    pub async fn await_touch_slop_or_cancellation(
        &mut self,
        pass: PointerEventPass,
        down: &PointerDown,
        slop: f32,
    ) -> Option<(PointerChange, f32, f32)> {
        let start = down.position?;
        loop {
            let event = self.await_pointer_event(pass).await;
            if event.is_cancelled() {
                return None;
            }
            for change in event.changes {
                if change.pointer_id != down.change.pointer_id {
                    continue;
                }
                if change.is_consumed() {
                    return None;
                }
                match change.content {
                    CursorEventContent::Moved(position) => {
                        let dx = position.x - start.x;
                        let dy = position.y - start.y;
                        if dx.hypot(dy) >= slop {
                            return Some((change, dx, dy));
                        }
                    }
                    CursorEventContent::Released(_)
                    | CursorEventContent::Cancelled(_)
                    | CursorEventContent::Scroll(_) => {
                        return None;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Waits for a release, cancellation, or a frame-clock timeout.
    pub async fn await_long_press_or_cancellation(
        &mut self,
        pass: PointerEventPass,
        down: &PointerDown,
        timeout: std::time::Duration,
        slop: f32,
    ) -> LongPressResult {
        let queue = self.queue.clone();
        let mut position = down.position;
        let mut event: Pin<Box<dyn Future<Output = PointerEvent>>> = Box::pin(AwaitPointerEvent {
            queue: queue.clone(),
            pass,
        });
        let mut frame = next_frame();
        loop {
            match select(event, frame).await {
                Either::Left((next_event, next_frame)) => {
                    frame = next_frame;
                    if next_event.is_cancelled() {
                        return LongPressResult::Cancelled;
                    }
                    let within = next_event.is_pointer_inside();
                    let next_pass = if next_event.pass == PointerEventPass::Final {
                        pass
                    } else {
                        PointerEventPass::Final
                    };
                    for change in next_event.changes {
                        if change.pointer_id != down.change.pointer_id {
                            continue;
                        }
                        if change.is_consumed() || !within {
                            return LongPressResult::Cancelled;
                        }
                        if let Some(point) = change.position() {
                            position = Some(point);
                            if down
                                .position
                                .is_some_and(|start| start.distance_to(point) > slop)
                            {
                                return LongPressResult::Cancelled;
                            }
                        }
                        match change.content {
                            CursorEventContent::Released(_) => {
                                return LongPressResult::Released(change);
                            }
                            CursorEventContent::Cancelled(_) => {
                                return LongPressResult::Cancelled;
                            }
                            CursorEventContent::Scroll(_) => {
                                return LongPressResult::Cancelled;
                            }
                            _ => {}
                        }
                    }
                    event = Box::pin(AwaitPointerEvent {
                        queue: queue.clone(),
                        pass: next_pass,
                    });
                }
                Either::Right((now, next_event)) => {
                    if now.saturating_duration_since(down.change.timestamp) >= timeout {
                        return LongPressResult::Triggered { position };
                    }
                    event = next_event;
                    frame = next_frame();
                }
            }
        }
    }

    /// Runs `detector` repeatedly, waiting for all observed buttons and pointers
    /// to lift before starting the next gesture.
    pub async fn await_each_gesture<F, Fut>(&mut self, mut detector: F)
    where
        F: FnMut(PointerInputScope) -> Fut,
        Fut: Future<Output = ()>,
    {
        loop {
            detector(PointerInputScope {
                queue: self.queue.clone(),
            })
            .await;
            if self.queue.cancelled.get() {
                return;
            }
            while !self.queue.pressed.borrow().is_empty() {
                if self
                    .await_pointer_event(PointerEventPass::Final)
                    .await
                    .is_cancelled()
                {
                    return;
                }
            }
        }
    }
}

fn next_frame() -> Pin<Box<dyn Future<Output = crate::time::Instant>>> {
    Box::pin(crate::with_frame_time(|now| now))
}

struct AwaitPointerEvent {
    queue: Rc<PointerEventQueue>,
    pass: PointerEventPass,
}

impl Drop for AwaitPointerEvent {
    fn drop(&mut self) {
        self.queue.pass.set(None);
        self.queue.waiter.borrow_mut().take();
    }
}

impl Future for AwaitPointerEvent {
    type Output = PointerEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.queue.cancelled.get() {
            return Poll::Ready(PointerEvent::cancelled(this.pass));
        }
        if let Some(event) = this.queue.event.borrow_mut().take() {
            return Poll::Ready(event);
        }
        this.queue.pass.set(Some(this.pass));
        *this.queue.waiter.borrow_mut() = Some(cx.waker().clone());
        Poll::Pending
    }
}

struct PointerInputRuntime<K> {
    key: K,
    queue: Rc<PointerEventQueue>,
    task: Rc<RefCell<Option<crate::Task>>>,
}

impl<K: PartialEq> PointerInputRuntime<K> {
    fn restart<F, Fut>(&mut self, key: K, detector: &F)
    where
        F: Fn(PointerInputScope) -> Fut,
        Fut: Future<Output = ()> + 'static,
    {
        self.queue.cancel();
        self.task.borrow_mut().take();
        self.queue = Rc::new(PointerEventQueue {
            event: RefCell::new(None),
            pressed: RefCell::new(Vec::new()),
            waiter: RefCell::new(None),
            pass: Cell::new(None),
            cancelled: Cell::new(false),
        });
        self.key = key;
        self.task = Rc::new(RefCell::new(Some(crate::spawn_local(detector(
            PointerInputScope::new(self.queue.clone()),
        )))));
    }
}

pub(crate) struct PointerInputSource<K, F> {
    key: K,
    detector: Rc<F>,
}

impl<K, F> PointerInputSource<K, F> {
    pub(crate) fn new(key: K, detector: F) -> Self {
        Self {
            key,
            detector: Rc::new(detector),
        }
    }
}

impl<K, F, Fut> BuildModifierNode for PointerInputSource<K, F>
where
    K: PartialEq + Clone + 'static,
    F: Fn(PointerInputScope) -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    fn materialize(&self, context: &mut ModifierBuildContext) -> Option<Modifier> {
        let key = self.key.clone();
        let detector = self.detector.clone();
        let state = context.state(|| {
            let queue = Rc::new(PointerEventQueue {
                event: RefCell::new(None),
                pressed: RefCell::new(Vec::new()),
                waiter: RefCell::new(None),
                pass: Cell::new(None),
                cancelled: Cell::new(false),
            });
            let task = crate::spawn_local(detector(PointerInputScope::new(queue.clone())));
            PointerInputRuntime {
                key: key.clone(),
                queue,
                task: Rc::new(RefCell::new(Some(task))),
            }
        });
        state.with_mut(|runtime| {
            if runtime.key != key {
                runtime.restart(key.clone(), detector.as_ref());
            }
        });
        let queue = state.with(|runtime| runtime.queue.clone());
        let task = state.with(|runtime| runtime.task.clone());
        Some(
            Modifier::new()
                .push_pointer_preview_input(PointerInputTask {
                    task: task.clone(),
                    queue: queue.clone(),
                })
                .push_pointer_input(PointerInputTask {
                    task: task.clone(),
                    queue: queue.clone(),
                })
                .push_pointer_final_input(PointerInputTask { task, queue }),
        )
    }
}

struct PointerInputTask {
    task: Rc<RefCell<Option<crate::Task>>>,
    queue: Rc<PointerEventQueue>,
}

impl PointerInputModifierNode for PointerInputTask {
    fn receives_consumed(&self) -> bool {
        true
    }

    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        if input.pass == PointerEventPass::Initial {
            let mut pressed = self.queue.pressed.borrow_mut();
            for change in input.pointer_changes.iter() {
                match change.content {
                    CursorEventContent::Pressed(button) => {
                        let key = (change.pointer_id, button);
                        if !pressed.contains(&key) {
                            pressed.push(key);
                        }
                    }
                    CursorEventContent::Released(button)
                    | CursorEventContent::Cancelled(button) => {
                        pressed.retain(|key| *key != (change.pointer_id, button));
                    }
                    _ => {}
                }
            }
        }
        let task = self.task.borrow();
        let Some(task) = task.as_ref() else {
            return;
        };
        crate::async_dispatch::poll_now(task);
        if self.queue.pass.get() != Some(input.pass) {
            return;
        }
        let commands = Rc::new(PointerDispatchCommands::default());
        let event = PointerEvent {
            pass: input.pass,
            changes: input.pointer_changes.clone(),
            cursor_position: input.cursor_position_rel,
            window_cursor_position: input.cursor_position_abs(),
            size: crate::PxSize::new(input.computed_data.width, input.computed_data.height),
            local_to_window: input.local_to_window,
            key_modifiers: input.key_modifiers,
            commands: commands.clone(),
            cancelled: false,
        };
        self.queue.publish(event);
        crate::async_dispatch::poll_now(task);
        self.queue.event.borrow_mut().take();
        if commands.drag_window.get() {
            input.drag_window();
        }
        if commands.block_cursor.get() {
            input.block_cursor();
        }
        for change in input.pointer_changes.iter_mut() {
            if let Some(remainder) = commands.scroll_remainders.borrow().get(&change.pointer_id)
                && matches!(change.content, CursorEventContent::Scroll(_))
            {
                change.content = CursorEventContent::Scroll(remainder.clone());
            }
        }
        if let Some((request, show)) = commands.ime_request.borrow_mut().take() {
            let mut ime = input.ime_session();
            if show {
                ime.show(request);
            } else {
                ime.update(request);
            }
        }
    }

    fn on_detach(&self, _context: crate::modifier::ModifierNodeContext) {
        self.queue.cancel();
        self.task.borrow_mut().take();
    }
}
