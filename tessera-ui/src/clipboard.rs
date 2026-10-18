//! Focused clipboard input and platform replies.
//!
//! ## Usage
//!
//! Route browser or platform clipboard actions through the focused editor.

use std::{cell::RefCell, collections::VecDeque};

use crate::{keyboard_state::next_input_sequence, runtime::schedule_runtime_redraw};

/// An explicit clipboard action initiated by the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardAction {
    /// Copies the current selection.
    Copy,
    /// Copies and then removes the current selection after a successful write.
    Cut,
    /// Inserts plain text supplied by the platform paste event.
    Paste(String),
}

/// A clipboard write awaiting acknowledgement from the platform.
pub struct ClipboardWrite {
    text: String,
    on_success: Box<dyn FnOnce() + Send + Sync>,
}

impl ClipboardWrite {
    /// Returns the plain text to write.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Completes this write, applying its edit only after success.
    pub fn complete(self, succeeded: bool) {
        if succeeded {
            (self.on_success)();
        }
    }
}

/// The focused editor's response to a clipboard action.
pub enum ClipboardResponse {
    /// No focused editor accepted the action.
    Ignored,
    /// The action was handled without a clipboard write.
    Handled,
    /// The platform must write text and acknowledge its result.
    Write(ClipboardWrite),
}

/// Clipboard input routed to a node on the focused path.
pub struct ClipboardInput<'a> {
    /// The requested copy, cut, or paste action.
    pub action: &'a ClipboardAction,
    pub(crate) response: &'a mut Option<ClipboardResponse>,
}

impl ClipboardInput<'_> {
    /// Consumes the action without requesting a platform write.
    pub fn handle(&mut self) {
        *self.response = Some(ClipboardResponse::Handled);
    }

    /// Requests a write and a callback after the platform confirms success.
    pub fn write_text(&mut self, text: String, on_success: impl FnOnce() + Send + Sync + 'static) {
        *self.response = Some(ClipboardResponse::Write(ClipboardWrite {
            text,
            on_success: Box::new(on_success),
        }));
    }
}

/// Node-local clipboard handling for focused editors.
pub trait ClipboardInputModifierNode: Send + Sync + 'static {
    /// Handles a clipboard action on the focused path.
    fn on_clipboard_input(&self, input: ClipboardInput<'_>);
}

pub(crate) struct QueuedClipboardAction {
    pub sequence: u64,
    pub action: ClipboardAction,
    pub reply: Box<dyn FnOnce(ClipboardResponse) + Send + Sync>,
}

thread_local! {
    static REQUESTS: RefCell<VecDeque<QueuedClipboardAction>> = const { RefCell::new(VecDeque::new()) };
}

/// Queues a user clipboard action in order with keyboard input.
///
/// Platform integrations supply paste text from the clipboard event and handle
/// write responses using their platform clipboard service.
/// Call this on the runtime's UI thread.
pub fn request_clipboard_action(
    action: ClipboardAction,
    reply: impl FnOnce(ClipboardResponse) + Send + Sync + 'static,
) {
    REQUESTS.with_borrow_mut(|requests| {
        requests.push_back(QueuedClipboardAction {
            sequence: next_input_sequence(),
            action,
            reply: Box::new(reply),
        })
    });
    schedule_runtime_redraw();
}

pub(crate) fn take_requests() -> Vec<QueuedClipboardAction> {
    REQUESTS.with_borrow_mut(|requests| requests.drain(..).collect())
}

pub(crate) fn has_pending_requests() -> bool {
    REQUESTS.with_borrow(|requests| !requests.is_empty())
}

pub(crate) fn clear_requests() {
    let requests = REQUESTS.with_borrow_mut(std::mem::take);
    drop(requests);
}
