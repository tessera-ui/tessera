//! Keyboard events with their original modifier state.
//!
//! ## Usage
//!
//! Preserve keyboard shortcuts during slow frames.

use std::collections::VecDeque;

use winit::keyboard::ModifiersState;

#[derive(Debug)]
pub(crate) struct QueuedKeyEvent<E = winit::event::KeyEvent> {
    pub event: E,
    pub modifiers: ModifiersState,
}

#[derive(Debug)]
pub(crate) struct KeyboardQueue<E> {
    events: VecDeque<QueuedKeyEvent<E>>,
    modifiers: ModifiersState,
}

pub(crate) type KeyboardState = KeyboardQueue<winit::event::KeyEvent>;

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
        self.events.push_back(QueuedKeyEvent {
            event,
            modifiers: self.modifiers,
        });
    }

    pub fn take_events(&mut self) -> Vec<QueuedKeyEvent<E>> {
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
