//! Keyboard events with their original modifier state.
//!
//! ## Usage
//!
//! Preserve keyboard shortcuts during slow frames.

use std::collections::VecDeque;

use winit::keyboard::ModifiersState;

/// An owned keyboard sample accepted from a window or synthetic input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardEvent {
    /// Physical keyboard position when known.
    pub physical_key: winit::keyboard::PhysicalKey,
    /// Key meaning after the platform keyboard layout is applied.
    pub logical_key: winit::keyboard::Key,
    /// Text produced by this sample, if any.
    pub text: Option<String>,
    /// Standard, left, right, or numeric keypad location.
    pub location: winit::keyboard::KeyLocation,
    /// Whether the key was pressed or released.
    pub state: winit::event::ElementState,
    /// Whether this is a repeated press.
    pub repeat: bool,
}

impl KeyboardEvent {
    /// Creates a sample for a named navigation or command key.
    pub fn named(key: winit::keyboard::NamedKey, state: winit::event::ElementState) -> Self {
        use winit::keyboard::{KeyCode, NamedKey, NativeKeyCode, PhysicalKey};
        let code = match key {
            NamedKey::ArrowLeft => Some(KeyCode::ArrowLeft),
            NamedKey::ArrowRight => Some(KeyCode::ArrowRight),
            NamedKey::ArrowUp => Some(KeyCode::ArrowUp),
            NamedKey::ArrowDown => Some(KeyCode::ArrowDown),
            NamedKey::Tab => Some(KeyCode::Tab),
            NamedKey::Enter => Some(KeyCode::Enter),
            NamedKey::Escape => Some(KeyCode::Escape),
            NamedKey::Space => Some(KeyCode::Space),
            NamedKey::Backspace => Some(KeyCode::Backspace),
            NamedKey::Delete => Some(KeyCode::Delete),
            NamedKey::Home => Some(KeyCode::Home),
            NamedKey::End => Some(KeyCode::End),
            NamedKey::PageUp => Some(KeyCode::PageUp),
            NamedKey::PageDown => Some(KeyCode::PageDown),
            _ => None,
        };
        Self {
            physical_key: code
                .map(PhysicalKey::Code)
                .unwrap_or(PhysicalKey::Unidentified(NativeKeyCode::Unidentified)),
            logical_key: winit::keyboard::Key::Named(key),
            text: None,
            location: winit::keyboard::KeyLocation::Standard,
            state,
            repeat: false,
        }
    }
}

impl From<winit::event::KeyEvent> for KeyboardEvent {
    fn from(event: winit::event::KeyEvent) -> Self {
        Self {
            physical_key: event.physical_key,
            logical_key: event.logical_key,
            text: event.text.map(|text| text.to_string()),
            location: event.location,
            state: event.state,
            repeat: event.repeat,
        }
    }
}

#[derive(Debug)]
pub(crate) struct QueuedInput<E = KeyboardEvent> {
    pub event: E,
    pub modifiers: ModifiersState,
    pub sequence: u64,
    pub timestamp: crate::time::Instant,
}

pub(crate) fn next_input_sequence() -> u64 {
    #[derive(Default)]
    struct Sequence(std::cell::Cell<u64>);
    let sequence = crate::local_runtime_resource::<Sequence>();
    let next = sequence
        .0
        .get()
        .checked_add(1)
        .expect("input sequence exhausted");
    sequence.0.set(next);
    next
}

#[derive(Debug)]
pub(crate) struct KeyboardQueue<E> {
    events: VecDeque<QueuedInput<E>>,
    modifiers: ModifiersState,
}

pub(crate) type KeyboardState = KeyboardQueue<KeyboardEvent>;

impl<E> Default for KeyboardQueue<E> {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            modifiers: ModifiersState::empty(),
        }
    }
}

impl<E> KeyboardQueue<E> {
    pub fn push_event(&mut self, event: E) {
        self.push_event_at(event, crate::time::Instant::now());
    }

    pub fn push_event_at(&mut self, event: E, timestamp: crate::time::Instant) {
        self.events.push_back(QueuedInput {
            event,
            modifiers: self.modifiers,
            sequence: next_input_sequence(),
            timestamp,
        });
    }

    pub fn take_events(&mut self) -> Vec<QueuedInput<E>> {
        self.events.drain(..).collect()
    }

    pub fn update_modifiers(&mut self, new_state: ModifiersState) {
        self.modifiers = new_state;
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_release_does_not_rewrite_queued_shortcuts() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut queue = KeyboardQueue::default();
        queue.update_modifiers(ModifiersState::SHIFT);
        queue.push_event("Tab pressed");
        queue.push_event("Tab released");
        queue.update_modifiers(ModifiersState::empty());
        queue.push_event("Tab pressed again");
        let events = queue.take_events();
        assert_eq!(
            events.iter().map(|event| event.event).collect::<Vec<_>>(),
            ["Tab pressed", "Tab released", "Tab pressed again"]
        );
        assert_eq!(
            events
                .iter()
                .map(|event| event.modifiers)
                .collect::<Vec<_>>(),
            [
                ModifiersState::SHIFT,
                ModifiersState::SHIFT,
                ModifiersState::empty()
            ]
        );
        assert!(queue.take_events().is_empty());
    }

    #[test]
    fn long_frames_preserve_order_and_each_modifier_combination() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut queue = KeyboardQueue::default();
        for index in 0..512 {
            queue.update_modifiers(if index % 2 == 0 {
                ModifiersState::CONTROL | ModifiersState::SHIFT
            } else {
                ModifiersState::ALT
            });
            queue.push_event(index);
        }
        queue.update_modifiers(ModifiersState::empty());
        let events = queue.take_events();
        assert_eq!(events.len(), 512);
        for (index, event) in events.iter().enumerate() {
            assert_eq!(event.event, index);
            assert_eq!(
                event.modifiers,
                if index % 2 == 0 {
                    ModifiersState::CONTROL | ModifiersState::SHIFT
                } else {
                    ModifiersState::ALT
                }
            );
        }
    }
}
