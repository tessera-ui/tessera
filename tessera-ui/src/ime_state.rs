pub(crate) type ImeEvent = (Option<crate::focus::FocusHandleId>, winit::event::Ime);

#[derive(Default)]
pub(crate) struct ImeState {
    events: crate::keyboard_state::KeyboardQueue<ImeEvent>,
}

impl ImeState {
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn push_event(&mut self, event: winit::event::Ime) {
        self.push_event_for(None, event);
    }

    pub fn push_event_for(
        &mut self,
        client: Option<crate::focus::FocusHandleId>,
        event: winit::event::Ime,
    ) {
        self.events.push_event((client, event));
    }

    pub fn take_events(&mut self) -> Vec<crate::keyboard_state::QueuedInput<ImeEvent>> {
        self.events.take_events()
    }
}
